use wgpu::util::DeviceExt;

pub struct ComputeEngine {
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipeline: wgpu::ComputePipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
    // Buffers
    particle_buffer_a: wgpu::Buffer,
    particle_buffer_b: wgpu::Buffer,
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
    g: f32,
    count: u32,
    _pad: u32,
}

impl ComputeEngine {
    pub async fn new(device: wgpu::Device, queue: wgpu::Queue, count: u32) -> Self {
        // 1. Create Buffers
        let particle_size = std::mem::size_of::<Particle>() as u64;
        let buf_size = particle_size * count as u64;
        
        let particle_buffer_a = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Particle Buffer A"),
            size: buf_size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let particle_buffer_b = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Particle Buffer B"),
            size: buf_size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let sim_params = SimParams { dt: 0.016, g: 1.0, count, _pad: 0 };
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Sim Params Buffer"),
            contents: bytemuck::cast_slice(&[sim_params]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // 2. Shader
        let shader = device.create_shader_module(wgpu::include_wgsl!("nbody.wgsl"));

        // 3. Pipeline
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
            label: None,
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Compute Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Compute Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: "main",
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: particle_buffer_a.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: particle_buffer_b.as_entire_binding(),
                },
            ],
            label: None,
        });

        Self {
            device,
            queue,
            pipeline,
            bind_group_layout,
            bind_group,
            particle_buffer_a,
            particle_buffer_b,
            uniform_buffer,
            particle_count: count,
        }
    }

    pub fn step(&mut self) {
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Compute Encoder"),
        });

        {
            let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Compute Pass"),
                timestamp_writes: None, 
            });
            cpass.set_pipeline(&self.pipeline);
            cpass.set_bind_group(0, &self.bind_group, &[]);
            let work_group_count = (self.particle_count as f32 / 256.0).ceil() as u32;
            cpass.dispatch_workgroups(work_group_count, 1, 1);
        }

        self.queue.submit(Some(encoder.finish()));

        // Ping-pong buffers so the next step reads the latest output.
        std::mem::swap(&mut self.particle_buffer_a, &mut self.particle_buffer_b);
        self.bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: self.particle_buffer_a.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: self.particle_buffer_b.as_entire_binding(),
                },
            ],
            label: Some("Compute Bind Group"),
        });
    }
}
