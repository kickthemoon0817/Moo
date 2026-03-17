use std::sync::Arc;
use std::sync::atomic::Ordering;
use crate::core::state::PhaseSpace;
use crate::platform::compute::{ComputeEngine, SimConfig};
use crate::control::{CommandQueue, SimCommand, SimMetrics};

/// Shared initialization parameters so `new()` and `reset()` produce identical initial state.
#[derive(Debug, Clone)]
pub struct SimInitConfig {
    pub n_particles: u32,
    pub spacing: f64,
    pub cols: usize,
    pub start_y: f64,
}

impl SimInitConfig {
    fn init_state(&self, state: &mut PhaseSpace) {
        for i in 0..self.n_particles as usize {
            let col = i % self.cols;
            let row = i / self.cols;
            state.q[i * 3] = (col as f64) * self.spacing - (self.cols as f64 * self.spacing / 2.0);
            state.q[i * 3 + 1] = self.start_y + (row as f64) * self.spacing;
            state.q[i * 3 + 2] = 0.0;

            state.v[i * 3] = 0.0;
            state.v[i * 3 + 1] = 0.0;
            state.v[i * 3 + 2] = 0.0;

            state.mass[i * 3] = 1.0;
            state.mass[i * 3 + 1] = 1.0;
            state.mass[i * 3 + 2] = 1.0;

            state.radius[i] = self.spacing / 2.0;
        }
    }
}

pub struct Simulation {
    pub state: PhaseSpace,
    pub compute: ComputeEngine,
    pub config: SimConfig,
    pub command_queue: Option<CommandQueue>,
    pub init_config: SimInitConfig,
    pub metrics: Option<Arc<SimMetrics>>,
    pub n_particles: u32,
    pub running: bool,
}

impl Simulation {
    pub async fn new(device: &wgpu::Device, n_particles: u32) -> Self {
        let init_config = SimInitConfig {
            n_particles,
            spacing: 15.0,
            cols: 64,
            start_y: -300.0,
        };

        let dof = n_particles as usize * 3;
        let mut state = PhaseSpace::new(dof);
        init_config.init_state(&mut state);

        let compute = ComputeEngine::new(device, n_particles).await;

        let config = SimConfig {
            dt: 0.005,
            h: 25.0,
            rho0: 0.01,
            stiffness: 2000.0,
            viscosity: 200.0,
            mouse_pos: [0.0, 0.0],
            mouse_pressed: false,
        };

        Self {
            state,
            compute,
            config,
            command_queue: None,
            init_config,
            metrics: None,
            n_particles,
            running: false,
        }
    }

    pub fn reset(&mut self, queue: &wgpu::Queue) {
        self.init_config.init_state(&mut self.state);

        self.compute
            .write_state(queue, &self.state.q, &self.state.v, &self.state.mass);

        if let Some(ref metrics) = self.metrics {
            metrics.step_count.store(0, Ordering::Relaxed);
        }
    }

    pub fn step(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        self.process_commands(device, queue);

        if self.running {
            self.compute.write_params(queue, self.config);

            for _ in 0..10 {
                self.compute.step(device, queue);
            }

            if let Some(ref metrics) = self.metrics {
                metrics.step_count.fetch_add(1, Ordering::Relaxed);
            }
        }
    }

    fn process_commands(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        if let Some(queue_ref) = self.command_queue.take() {
            while let Some(cmd) = queue_ref.try_recv() {
                match cmd {
                    SimCommand::Pause => self.running = false,
                    SimCommand::Resume => self.running = true,
                    SimCommand::Step(n) => {
                        self.compute.write_params(queue, self.config);
                        for _ in 0..n {
                            self.compute.step(device, queue);
                        }
                        if let Some(ref metrics) = self.metrics {
                            metrics.step_count.fetch_add(n as u64, Ordering::Relaxed);
                        }
                    },
                    SimCommand::SetDt(dt) => self.config.dt = dt,
                    SimCommand::SetGravity(_, _) => { /* TODO */ },
                    SimCommand::Reset => self.reset(queue),
                }
            }
            self.command_queue = Some(queue_ref);
        }
    }

    pub fn interact(&mut self, _queue: &wgpu::Queue, _mouse_pos: [f32; 2], _is_pressed: bool) {
        // Handled directly by compute.write_params in the specific app implementation (Khe).
        // This is kept here for future abstraction if needed.
    }

    pub async fn init_headless() -> (wgpu::Device, wgpu::Queue) {
        let instance = wgpu::Instance::default();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .expect("Failed to find adapter");

        adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("Headless Device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: wgpu::MemoryHints::default(),
                ..Default::default()
            })
            .await
            .expect("Failed to create device")
    }
}
