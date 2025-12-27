use wgpu::util::DeviceExt;

pub struct ComputeEngine {
    // SPH Pipelines
    density_pipeline: wgpu::ComputePipeline,
    force_pipeline: wgpu::ComputePipeline,
    // Grid Pipelines
    grid_indices_pipeline: wgpu::ComputePipeline,
    clear_offsets_pipeline: wgpu::ComputePipeline,
    find_offsets_pipeline: wgpu::ComputePipeline,
    sort_pipeline: wgpu::ComputePipeline,

    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
    _sort_bg_layout: wgpu::BindGroupLayout,
    sort_bind_group: wgpu::BindGroup,

    // Buffers
    particle_buffer_a: wgpu::Buffer,
    particle_buffer_b: wgpu::Buffer,
    density_buffer: wgpu::Buffer,
    grid_buffer: wgpu::Buffer,
    offset_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
    sort_params_buffer: wgpu::Buffer,

    particle_count: u32,
    grid_dim: u32,
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
    viscosity: f32,
    count: u32,
    grid_dim: u32,
    _pad0: u32, // Padding for vec2 alignment (8 bytes)
    mouse_pos: [f32; 2],
    mouse_pressed: u32, // 0 or 1
    _pad1: u32,
}

// ... helper for sort params
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct SortParams {
    num_elements: u32,
    block_height: u32,
    block_width: u32,
    algo: u32,
}

impl ComputeEngine {
    pub async fn new(device: &wgpu::Device, count: u32) -> Self {
        // 1. Create Buffers
        let particle_size = std::mem::size_of::<Particle>() as u64;
        let buf_size = particle_size * count as u64;
        let float_size = std::mem::size_of::<f32>() as u64;
        let density_size = float_size * count as u64;

        let grid_dim = 16384; // Hash table size
        let pair_size = 8; // u32, u32
        let grid_buf_size = pair_size * count as u64;
        let offset_buf_size = 4 * grid_dim as u64;

        // ... Buffers A/B exist ...
        let particle_buffer_a = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Particle Buffer A"),
            size: buf_size,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC
                | wgpu::BufferUsages::VERTEX,
            mapped_at_creation: false,
        });

        let particle_buffer_b = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Particle Buffer B"),
            size: buf_size,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC
                | wgpu::BufferUsages::VERTEX,
            mapped_at_creation: false,
        });

        let density_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Density Buffer"),
            size: density_size,
            usage: wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });

        let grid_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Grid Indices Buffer"),
            size: grid_buf_size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let offset_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Grid Offsets Buffer"),
            size: offset_buf_size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let sim_params = SimParams {
            dt: 0.005,
            h: 25.0,
            rho0: 0.01,
            stiffness: 2000.0,
            viscosity: 200.0,
            count,
            grid_dim,
            _pad0: 0,
            mouse_pos: [0.0, 0.0],
            mouse_pressed: 0,
            _pad1: 0,
        };

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Sim Params Buffer"),
            contents: bytemuck::cast_slice(&[sim_params]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // 2. Shaders
        let shader_sph = device.create_shader_module(wgpu::include_wgsl!("sph.wgsl"));
        let shader_grid = device.create_shader_module(wgpu::include_wgsl!("grid.wgsl"));
        let shader_sort = device.create_shader_module(wgpu::include_wgsl!("sort.wgsl"));

        // 3. Bind Group Layout
        // Main Layout (Simulation + Grid)
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
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 5,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
            label: Some("Main Bind Group Layout"),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Main Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let density_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Density"),
            layout: Some(&pipeline_layout),
            module: &shader_sph,
            entry_point: Some("calc_density"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });
        let force_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Force"),
            layout: Some(&pipeline_layout),
            module: &shader_sph,
            entry_point: Some("calc_force"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });
        let grid_indices_pipeline =
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Grid Indices"),
                layout: Some(&pipeline_layout),
                module: &shader_grid,
                entry_point: Some("calc_grid_indices"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                cache: None,
            });
        let clear_offsets_pipeline =
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Clear Offsets"),
                layout: Some(&pipeline_layout),
                module: &shader_grid,
                entry_point: Some("clear_offsets"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                cache: None,
            });
        let find_offsets_pipeline =
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Find Offsets"),
                layout: Some(&pipeline_layout),
                module: &shader_grid,
                entry_point: Some("find_offsets"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                cache: None,
            });

        // Sort Layout (Simplified)
        let sort_bg_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
            label: Some("Sort Bind Group Layout"),
        });
        let sort_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Sort Pipeline Layout"),
            bind_group_layouts: &[&sort_bg_layout],
            push_constant_ranges: &[],
        });
        let sort_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Sort"),
            layout: Some(&sort_pipeline_layout),
            module: &shader_sort,
            entry_point: Some("bitonic_sort"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
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
                    resource: density_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: particle_buffer_b.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: grid_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: offset_buffer.as_entire_binding(),
                },
            ],
            label: Some("Main Bind Group"),
        });

        // Sort Bind Group
        // Dynamic Offset for Params (256 byte alignment)
        let sort_params_align = 256;
        let max_passes = 100;
        let sort_params_size = (sort_params_align * max_passes) as u64;

        let sort_params_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Sort Params Buffer"),
            size: sort_params_size,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let sort_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &sort_bg_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &sort_params_buffer,
                        offset: 0,
                        size: wgpu::BufferSize::new(std::mem::size_of::<SortParams>() as u64),
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: grid_buffer.as_entire_binding(),
                },
            ],
            label: Some("Sort Bind Group"),
        });

        Self {
            density_pipeline,
            force_pipeline,
            grid_indices_pipeline,
            clear_offsets_pipeline,
            find_offsets_pipeline,
            sort_pipeline,
            bind_group_layout,
            bind_group,
            _sort_bg_layout: sort_bg_layout,
            sort_bind_group,
            particle_buffer_a,
            particle_buffer_b,
            density_buffer,
            grid_buffer,
            offset_buffer,
            uniform_buffer,
            sort_params_buffer,
            particle_count: count,
            grid_dim,
        }
    }

    pub fn step(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("SPH Encoder"),
        });

        let work_group_count = (self.particle_count as f32 / 256.0).ceil() as u32;

        // 1. Grid Indices
        {
            let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Grid Indices"),
                timestamp_writes: None,
            });
            cpass.set_bind_group(0, &self.bind_group, &[]);
            cpass.set_pipeline(&self.grid_indices_pipeline);
            cpass.dispatch_workgroups(work_group_count, 1, 1);
        }

        // 2. Sort (Bitonic)
        // Calculate passes
        let mut n = 1u32;
        while n < self.particle_count {
            n *= 2;
        } // Next POT

        let mut params = Vec::new();
        let mut offsets = Vec::new();
        let align = 256;

        let mut k = 2u32;
        while k <= n {
            let mut j = k / 2;
            while j > 0 {
                params.push(SortParams {
                    num_elements: self.particle_count,
                    block_height: k,
                    block_width: j,
                    algo: 0,
                });
                offsets.push((params.len() - 1) as u32 * align);
                j /= 2;
            }
            k *= 2;
        }

        // Write Sort Params
        let mut raw_bytes = Vec::with_capacity(params.len() * align as usize);
        for p in &params {
            let bytes = bytemuck::bytes_of(p);
            raw_bytes.extend_from_slice(bytes);
            // Pad
            let pad = align as usize - bytes.len();
            raw_bytes.extend(std::iter::repeat_n(0, pad));
        }
        queue.write_buffer(&self.sort_params_buffer, 0, &raw_bytes);

        // Dispatch Sort Loops
        {
            let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Bitonic Sort"),
                timestamp_writes: None,
            });
            cpass.set_pipeline(&self.sort_pipeline);

            for (i, _) in params.iter().enumerate() {
                let offset = i as u32 * align;
                cpass.set_bind_group(0, &self.sort_bind_group, &[offset]);
                cpass.dispatch_workgroups(work_group_count, 1, 1);
                // Need global memory barrier between passes?
                // Compute Passes in WGPU process strictly in order, but memory visibility?
                // Storage Buffer Read/Write dependency.
                // WGPU normally requires separate dispatch calls.
                // In same pass, dispatch barrier?
                // Safest: Use separate passes if we fear race, but standard is single pass set_pipeline loop.
                // "dispatch_workgroups" acts as a barrier for subsequent dispatches in same pass FOR UAV?
                // No, standard Vulkan/D3D12 does not guarantee UAV visibility without barrier.
                // WGPU might insert barriers if resources are tracked.
                // Let's rely on WGPU tracking.
            }
        }

        // 3. Clear Offsets
        let grid_wg = (self.grid_dim as f32 / 256.0).ceil() as u32;
        {
            let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Clear Offsets"),
                timestamp_writes: None,
            });
            cpass.set_bind_group(0, &self.bind_group, &[]);
            cpass.set_pipeline(&self.clear_offsets_pipeline);
            cpass.dispatch_workgroups(grid_wg, 1, 1);
        }

        // 4. Find Offsets
        {
            let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Find Offsets"),
                timestamp_writes: None,
            });
            cpass.set_bind_group(0, &self.bind_group, &[]);
            cpass.set_pipeline(&self.find_offsets_pipeline);
            cpass.dispatch_workgroups(work_group_count, 1, 1);
        }

        // 5. Density
        {
            let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("SPH Density"),
                timestamp_writes: None,
            });
            cpass.set_bind_group(0, &self.bind_group, &[]);
            cpass.set_pipeline(&self.density_pipeline);
            cpass.dispatch_workgroups(work_group_count, 1, 1);
        }

        // 6. Force
        {
            let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("SPH Force"),
                timestamp_writes: None,
            });
            cpass.set_bind_group(0, &self.bind_group, &[]);
            cpass.set_pipeline(&self.force_pipeline);
            cpass.dispatch_workgroups(work_group_count, 1, 1);
        }

        queue.submit(Some(encoder.finish()));

        // Ping-pong buffers
        std::mem::swap(&mut self.particle_buffer_a, &mut self.particle_buffer_b);

        // Re-create Main Bind Group for next direction
        self.bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: self.particle_buffer_a.as_entire_binding(),
                }, // New Src
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: self.density_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: self.particle_buffer_b.as_entire_binding(),
                }, // New Dst
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: self.grid_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: self.offset_buffer.as_entire_binding(),
                },
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
                pos: [
                    q[idx] as f32,
                    q[idx + 1] as f32,
                    q[idx + 2] as f32,
                    mass[i * m_stride] as f32,
                ],
                vel: [v[idx] as f32, v[idx + 1] as f32, v[idx + 2] as f32, 0.0],
            });
        }
        // Write to current read source
        queue.write_buffer(&self.particle_buffer_a, 0, bytemuck::cast_slice(&data));
    }

    pub fn write_params(
        &self,
        queue: &wgpu::Queue,
        dt: f32,
        h: f32,
        rho0: f32,
        stiffness: f32,
        viscosity: f32,
        mouse_pos: [f32; 2],
        mouse_pressed: bool,
    ) {
        let params = SimParams {
            dt,
            h,
            rho0,
            stiffness,
            viscosity,
            count: self.particle_count,
            grid_dim: self.grid_dim,
            _pad0: 0,
            mouse_pos,
            mouse_pressed: if mouse_pressed { 1 } else { 0 },
            _pad1: 0,
        };
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[params]));
    }
}
