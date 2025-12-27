struct SimParams {
    dt: f32,
    h: f32,
    rho0: f32,
    stiffness: f32,
    viscosity: f32,
    count: u32,
    // Add grid params
    grid_dim: u32, // e.g., 128 (128x128x128 grid?) Or just Infinite Hashing?
    _pad: u32,
}

struct GridPair {
    cell_id: u32,
    particle_id: u32,
}

struct Particle {
    pos: vec4<f32>,
    vel: vec4<f32>,
}

@group(0) @binding(0) var<uniform> params: SimParams;
@group(0) @binding(1) var<storage, read> particles: array<Particle>;
@group(0) @binding(2) var<storage, read_write> grid_pairs: array<GridPair>;
@group(0) @binding(3) var<storage, read_write> cell_offsets: array<u32>; // value = start_index

// Helpers
fn get_cell_id(pos: vec3<f32>, h: f32) -> u32 {
    // Basic Spatial Hashing using extremely large primes for demo
    // Or just a simple grid if bounded.
    // Let's assume bounded box for demo: [-500, 500] in all axes?
    // h = 25.0. 
    // Grid coordinate = floor(pos / h)
    
    let grid_h = h;
    let cell = vec3<i32>(floor(pos / grid_h));
    
    // Hash function: (x * p1 XOR y * p2 XOR z * p3) % n
    let p1 = 73856093u;
    let p2 = 19349663u;
    let p3 = 83492791u;
    
    let n = params.grid_dim; // Size of hash table
    
    // Handle Negatives by bit casting? Or offsetting.
    // Offsetting is safer for visualization stability.
    // Simulation bounds ~[-250, 250]. +1000 to be safe positive.
    let off = 1000.0;
    
    let xi = u32(i32(cell.x + i32(off/h)));
    let yi = u32(i32(cell.y + i32(off/h)));
    let zi = u32(i32(cell.z + i32(off/h)));
    
    return ((xi * p1) ^ (yi * p2) ^ (zi * p3)) % n;
}

@compute @workgroup_size(256)
fn calc_grid_indices(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let i = global_id.x;
    if (i >= params.count) { return; }

    let p = particles[i];
    let cell_id = get_cell_id(p.pos.xyz, params.h);

    grid_pairs[i] = GridPair(cell_id, i);
}

@compute @workgroup_size(256)
fn clear_offsets(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let i = global_id.x;
    if (i >= params.grid_dim) { return; }
    cell_offsets[i] = 0xFFFFFFFFu; // Sentinel
}

@compute @workgroup_size(256)
fn find_offsets(@builtin(global_invocation_id) global_id: vec3<u32>) {
    // Run over sorted grid_pairs.
    // If pair[i].cell_id != pair[i-1].cell_id, then i is the start of a new cell.
    
    let i = global_id.x;
    if (i >= params.count) { return; }
    
    let cell = grid_pairs[i].cell_id;
    let prev_cell = select(0xFFFFFFFFu, grid_pairs[i - 1].cell_id, i > 0u);
    
    if (cell != prev_cell) {
        cell_offsets[cell] = i;
    }
}
