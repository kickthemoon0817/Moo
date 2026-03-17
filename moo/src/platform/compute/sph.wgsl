struct SimParams {
    dt: f32,
    h: f32,       // Smoothing length
    rho0: f32,    // Rest density
    stiffness: f32, // Tait stiffness (B)
    viscosity: f32,
    count: u32,
    grid_dim: u32,
    mouse_pos: vec2<f32>,
    mouse_pressed: u32,
    _pad: u32,
}

struct Particle {
    pos: vec4<f32>, // xyz, w=mass
    vel: vec4<f32>, // xyz
}

struct GridPair {
    cell_id: u32,
    particle_id: u32,
}

@group(0) @binding(0) var<uniform> params: SimParams;
@group(0) @binding(1) var<storage, read> particlesSrc: array<Particle>;
@group(0) @binding(2) var<storage, read_write> density: array<f32>;
@group(0) @binding(3) var<storage, read_write> particlesDst: array<Particle>;
@group(0) @binding(4) var<storage, read_write> grid_pairs: array<GridPair>;
@group(0) @binding(5) var<storage, read_write> cell_offsets: array<u32>;

const PI: f32 = 3.1415926535;

// Poly6 Kernel for Density
fn poly6(r2: f32, h: f32) -> f32 {
    let h2 = h * h;
    if (r2 > h2) { return 0.0; }
    let term = h2 - r2;
    return (315.0 / (64.0 * PI * pow(h, 9.0))) * term * term * term;
}

// Spiky Gradient for Pressure
fn spiky_grad(r_vec: vec3<f32>, r: f32, h: f32) -> vec3<f32> {
    if (r > h || r < 1e-5) { return vec3<f32>(0.0); }
    let term = h - r;
    let scalar = -45.0 / (PI * pow(h, 6.0)) * term * term / r;
    return r_vec * scalar;
}

@compute @workgroup_size(256)
fn calc_density(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let i = global_id.x;
    if (i >= params.count) { return; }

    let pi = particlesSrc[i];
    let h = params.h;
    var rho = 0.0;
    
    // Grid Search
    // Iterate 3x3x3 cells around current position
    let center_cell = vec3<i32>(floor(pi.pos.xyz / h));
    
    for (var z = -1; z <= 1; z++) {
    for (var y = -1; y <= 1; y++) {
    for (var x = -1; x <= 1; x++) {
        // Re-hash neighbor cell
        let off = 1000.0;
        let cx = center_cell.x + x;
        let cy = center_cell.y + y;
        let cz = center_cell.z + z;
        
        // Hash
        let p1 = 73856093u;
        let p2 = 19349663u;
        let p3 = 83492791u;
        let n = params.grid_dim;
        
        let xi = u32(i32(cx + i32(off/h)));
        let yi = u32(i32(cy + i32(off/h)));
        let zi = u32(i32(cz + i32(off/h)));
        let cell_index = ((xi * p1) ^ (yi * p2) ^ (zi * p3)) % n;
        
        // Read Offsets
        let start_idx = cell_offsets[cell_index];
        
        if (start_idx != 0xFFFFFFFFu) {
            // Iterate particles in this cell
            var k = start_idx;
            loop {
                if (k >= params.count) { break; }
                let pair = grid_pairs[k];
                if (pair.cell_id != cell_index) { break; }
                
                let j = pair.particle_id;
                
                let pj = particlesSrc[j];
                let diff = pi.pos.xyz - pj.pos.xyz;
                let r2 = dot(diff, diff);
                
                if (r2 < h*h) {
                     rho += pj.pos.w * poly6(r2, h);
                }
                // ----------------------------
                
                k++;
            }
        }
    }
    }
    }

    density[i] = rho;
}

@compute @workgroup_size(256)
fn calc_force(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let i = global_id.x;
    if (i >= params.count) { return; }

    let pi = particlesSrc[i];
    let rho_i = density[i];
    let h = params.h;
    
    // Tait EOS: P = B * ((rho / rho0)^7 - 1)
    let p_i = params.stiffness * (pow(max(rho_i / params.rho0, 1.0), 7.0) - 1.0);

    var force_press = vec3<f32>(0.0);
    var force_visc = vec3<f32>(0.0);

    let center_cell = vec3<i32>(floor(pi.pos.xyz / h));
    
    for (var z = -1; z <= 1; z++) {
    for (var y = -1; y <= 1; y++) {
    for (var x = -1; x <= 1; x++) {
        let off = 1000.0;
        let cx = center_cell.x + x;
        let cy = center_cell.y + y;
        let cz = center_cell.z + z;
        
        let p1 = 73856093u;
        let p2 = 19349663u;
        let p3 = 83492791u;
        let n = params.grid_dim;
        
        let xi = u32(i32(cx + i32(off/h)));
        let yi = u32(i32(cy + i32(off/h)));
        let zi = u32(i32(cz + i32(off/h)));
        let cell_index = ((xi * p1) ^ (yi * p2) ^ (zi * p3)) % n;
        
        let start_idx = cell_offsets[cell_index];
        
        if (start_idx != 0xFFFFFFFFu) {
            var k = start_idx;
            loop {
                if (k >= params.count) { break; }
                let pair = grid_pairs[k];
                if (pair.cell_id != cell_index) { break; }
                
                let j = pair.particle_id;
                if (i == j) { k++; continue; } 
                
                let pj = particlesSrc[j];
                let diff = pi.pos.xyz - pj.pos.xyz;
                let r = length(diff);
                
                if (r < h) {
                    let rho_j = density[j];
                    let p_j = params.stiffness * (pow(max(rho_j / params.rho0, 1.0), 7.0) - 1.0);
                    
                    // Stability: Clamp density to avoid division by zero (though unlikely with proper H)
                    let safe_rho_i = max(rho_i, 0.0001);
                    let safe_rho_j = max(rho_j, 0.0001);

                    let term_p = (p_i / (safe_rho_i * safe_rho_i)) + (p_j / (safe_rho_j * safe_rho_j));
                    force_press -= pj.pos.w * term_p * spiky_grad(diff, r, h);

                    let term_v = (h - r);
                    let laplacian = (45.0 / (PI * pow(h, 6.0))) * term_v;
                    
                    let vel_diff = pj.vel.xyz - pi.vel.xyz;
                    force_visc += (params.viscosity / rho_j) * pj.pos.w * vel_diff * laplacian;
                }

                
                k++;
            }
        }
    }
    }
    }

    // --- Mouse Interaction ---
    var mouse_force = vec3<f32>(0.0);
    if (params.mouse_pressed != 0u) {
        let interaction_radius = 50.0;
        let spring_k = 2000.0; // Strong pull
        
        let delta = params.mouse_pos - pi.pos.xy;
        let dist = length(delta);
        
        if (dist < interaction_radius && dist > 1.0) {
             let dir = delta / dist;
             // Spring force: F = k * x
             // Pull towards mouse
             mouse_force += vec3<f32>(dir * spring_k * (dist / interaction_radius), 0.0);
             
             // Damping
             mouse_force -= pi.vel.xyz * 2.0; 
        }
    }

    let g = vec3<f32>(0.0, -500.0, 0.0);
    
    let mass_i = pi.pos.w;
    var accel = force_press + (force_visc / mass_i) + g + (mouse_force / mass_i); 
    
    // Stability: Clamp Acceleration
    let max_accel = 50000.0; 
    let accel_len = length(accel);
    if (accel_len > max_accel) {
        accel = (accel / accel_len) * max_accel;
    } 
    
    // Symplectic Euler
    let vel = pi.vel.xyz + accel * params.dt;
    var pos = pi.pos.xyz + vel * params.dt;

    // Floor Constraint
    if (pos.y < -200.0) {
        pos.y = -200.0;
        let damp = 0.5;
        particlesDst[i].vel = vec4<f32>(vel.x * 0.9, -vel.y * damp, vel.z * 0.9, 0.0);
    } else {
        particlesDst[i].vel = vec4<f32>(vel, 0.0);
    }

    particlesDst[i].pos = vec4<f32>(pos, mass_i);
}
