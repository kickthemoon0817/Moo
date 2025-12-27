struct SimParams {
    dt: f32,
    h: f32,       // Smoothing length
    rho0: f32,    // Rest density
    stiffness: f32, // Tait stiffness (B)
    count: u32,
}

struct Particle {
    pos: vec4<f32>, // xyz, w=mass
    vel: vec4<f32>, // xyz
}

@group(0) @binding(0) var<uniform> params: SimParams;
@group(0) @binding(1) var<storage, read> particlesSrc: array<Particle>;
@group(0) @binding(2) var<storage, read_write> density: array<f32>;
@group(0) @binding(3) var<storage, read_write> particlesDst: array<Particle>;

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

    for (var j = 0u; j < params.count; j++) {
        let pj = particlesSrc[j];
        let diff = pi.pos.xyz - pj.pos.xyz;
        let r2 = dot(diff, diff);
        
        // Mass = pj.pos.w (stored in w component)
        rho += pj.pos.w * poly6(r2, h);
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
    // Avoid negative pressure (clamping)
    let p_i = params.stiffness * (pow(max(rho_i / params.rho0, 1.0), 7.0) - 1.0);

    var force_press = vec3<f32>(0.0);
    var force_visc = vec3<f32>(0.0); // Placeholder for phase 8

    for (var j = 0u; j < params.count; j++) {
        if (i == j) { continue; }
        
        let pj = particlesSrc[j];
        let diff = pi.pos.xyz - pj.pos.xyz;
        let r = length(diff);
        
        if (r < h) {
            let rho_j = density[j];
            let p_j = params.stiffness * (pow(max(rho_j / params.rho0, 1.0), 7.0) - 1.0);
            
            // Symmetric pressure force
            // F_i = - sum m_j * (Pi/rho_i^2 + Pj/rho_j^2) * gradW
            // Simplification: Standard SPH form
            let term = (p_i / (rho_i * rho_i)) + (p_j / (rho_j * rho_j));
            force_press -= pj.pos.w * term * spiky_grad(diff, r, h);
        }
    }

    // Gravity
    let g = vec3<f32>(0.0, -500.0, 0.0); // Strong gravity for demo
    let mass_i = pi.pos.w;
    var total_force = force_press * mass_i + g * mass_i; 
    // F_pressure is actually force density in some derivations, but Spiky returns Force directly if careful. 
    // Wait, m * (P/rho^2) * gradW ==> Units: kg * (Pa / (kg/m^3)^2) * (1/m) = kg * (N/m^2 / kg^2/m^6) * 1/m = kg * N * m^4 / kg^2 * 1/m = N * m^3 / kg.
    // If we multiply by mass, we get Force? 
    // Standard Monaghan: dv/dt = - sum m (P/rho^2 + ...) gradW. 
    // So Force = mass * dv/dt. 
    // My spiky_grad returns gradient of W. 
    // term units: Pa / (kg/m^3)^2 = N/m^2 / (kg^2/m^6) = N m^4 / kg^2.
    // m_j units: kg.
    // term * m_j = N m^4 / kg.
    // spiky_grad units: 1/m^4.
    // Result: N / kg = Acceleration.
    // So force_press accumulation above (without mass_i multiplication) is ACCELERATION.
    
    let accel = force_press + g; // Pressure term provides acceleration directly.
    
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
