use wgpu::util::DeviceExt;

pub struct ComputeEngine {
    density_pipeline: wgpu::ComputePipeline,
    force_pipeline: wgpu::ComputePipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
    // Buffers
    particle_buffer_a: wgpu::Buffer,
    particle_buffer_b: wgpu::Buffer,
    density_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
    particle_count: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Particle {
    pos: [f32; 4], // x, y, z, mass
    vel: [f32; 4], // vx, vy, vz, padding
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct SimParams {
    dt: f32,
    h: f32,
    rho0: f32,
    stiffness: f32,
    count: u32,
    _pad: [u32; 3], // Padding to ensure 16-byte alignment validation if needed
}

impl ComputeEngine {
    pub async fn new(device: &wgpu::Device, count: u32) -> Self {
        // 1. Create Buffers
        let particle_size = std::mem::size_of::<Particle>() as u64;
        let buf_size = particle_size * count as u64;
        let float_size = std::mem::size_of::<f32>() as u64;
        let density_size = float_size * count as u64;
        
        // A/B Buffers for Ping-Pong (Positions/Velocities)
        let particle_buffer_a = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Particle Buffer A"),
            size: buf_size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::VERTEX,
            mapped_at_creation: false,
        });

        let particle_buffer_b = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Particle Buffer B"),
            size: buf_size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::VERTEX,
            mapped_at_creation: false,
        });

        // Density Buffer (Intermediate)
        let density_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Density Buffer"),
            size: density_size,
            usage: wgpu::BufferUsages::STORAGE, // Read/Write in compute
            mapped_at_creation: false,
        });

        // Uniforms: h=25.0, rho0=0.002, k=10000.0 (Matched window.rs CPU logic)
        // dt = 0.005 (Stable for SPH)
        let sim_params = SimParams { 
            dt: 0.005, 
            h: 25.0, 
            rho0: 0.002, 
            stiffness: 10000.0, 
            count, 
            _pad: [0; 3] 
        };
        
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Sim Params Buffer"),
            contents: bytemuck::cast_slice(&[sim_params]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // 2. Shader
        let shader = device.create_shader_module(wgpu::include_wgsl!("sph.wgsl"));

        // 3. Bind Group Layout
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry { // 0: Uniforms
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry { // 1: ParticlesSrc (Read)
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry { // 2: Density (Read/Write)
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry { // 3: ParticlesDst (Write)
                    binding: 3,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
            label: Some("SPH Bind Group Layout"),
        });

        // 4. Pipelines
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("SPH Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let density_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Density Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: "calc_density",
        });

        let force_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Force Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: "calc_force",
        });

        // 5. Initial Bind Group (A -> B)
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: uniform_buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: particle_buffer_a.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 2, resource: density_buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 3, resource: particle_buffer_b.as_entire_binding() },
            ],
            label: Some("SPH Bind Group A->B"),
        });

        Self {
            density_pipeline,
            force_pipeline,
            bind_group_layout,
            bind_group,
            particle_buffer_a,
            particle_buffer_b,
            density_buffer,
            uniform_buffer,
            particle_count: count,
        }
    }

    pub fn step(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("SPH Encoder"),
        });

        let work_group_count = (self.particle_count as f32 / 256.0).ceil() as u32;

        {
            let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("SPH Compute Pass"),
                timestamp_writes: None, 
            });
            cpass.set_bind_group(0, &self.bind_group, &[]);
            
            // Pass 1: Density
            cpass.set_pipeline(&self.density_pipeline);
            cpass.dispatch_workgroups(work_group_count, 1, 1);

            // Barrier implied between dispatches in same pass? 
            // In WGPU, memory hazards are handled by the driver if resources are used, but here Density is RW in both?
            // Wait, calc_density writes Density. calc_force reads Density.
            // Standard WGPU requires a pipeline barrier. Multi-dispatch inside one pass *does* guarantee execution order, 
            // but memory visibility requires a pipeline barrier if the target is STORAGE.
            // However, WGPU's `dispatch` calls in sequence are ordered. 
            // Safety: Splitting into two passes ensures visibility if we are paranoid, but single pass usually works for sequential kernels.
            // I'll stick to single pass for now.
            
            // Pass 2: Force & Integration
            cpass.set_pipeline(&self.force_pipeline);
            cpass.dispatch_workgroups(work_group_count, 1, 1);
        }

        queue.submit(Some(encoder.finish()));

        // Ping-pong buffers
        std::mem::swap(&mut self.particle_buffer_a, &mut self.particle_buffer_b);
        
        // Re-create bind group for next direction
        // Src: Buffer A (was destination), Dst: Buffer B (was source)
        self.bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: self.uniform_buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: self.particle_buffer_a.as_entire_binding() }, // New Src
                wgpu::BindGroupEntry { binding: 2, resource: self.density_buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 3, resource: self.particle_buffer_b.as_entire_binding() }, // New Dst
            ],
            label: Some("SPH Bind Group"),
        });
    }

    pub fn current_buffer(&self) -> &wgpu::Buffer {
        &self.particle_buffer_a
    }

    pub fn write_state(&self, queue: &wgpu::Queue, q: &[f64], v: &[f64], mass: &[f64]) {
        let count = self.particle_count as usize;
        let mut data = Vec::with_capacity(count);
        for i in 0..count {
            let idx = i * 3;
            let m_stride = if mass.len() == q.len() { 3 } else { 1 };
            
            data.push(Particle {
                pos: [q[idx] as f32, q[idx+1] as f32, q[idx+2] as f32, mass[i*m_stride] as f32],
                vel: [v[idx] as f32, v[idx+1] as f32, v[idx+2] as f32, 0.0],
            });
        }
        // Write to current read source
        queue.write_buffer(&self.particle_buffer_a, 0, bytemuck::cast_slice(&data));
    }
}
