struct SimParams {
    dt: f32,
    g: f32,
    count: u32,
    pad: u32,
};

struct Particle {
    pos: vec4<f32>, // xyz, mass
    vel: vec4<f32>, // xyz, pad
};

@group(0) @binding(0) var<uniform> params: SimParams;
@group(0) @binding(1) var<storage, read> particlesSrc: array<Particle>;
@group(0) @binding(2) var<storage, read_write> particlesDst: array<Particle>;

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index = global_id.x;
    if (index >= params.count) {
        return;
    }

    var p = particlesSrc[index];
    var force = vec3<f32>(0.0);

    // O(N^2) Gravity
    for (var i = 0u; i < params.count; i++) {
        if (i == index) {
            continue;
        }
        let other = particlesSrc[i];
        let diff = other.pos.xyz - p.pos.xyz;
        let dist_sq = dot(diff, diff) + 1.0; // Softening
        let dist = sqrt(dist_sq);
        let f = (params.g * p.pos.w * other.pos.w) / dist_sq;
        force += normalize(diff) * f;
    }

    // Symplectic Euler
    let accel = force / p.pos.w;
    let vel = p.vel.xyz + accel * params.dt;
    let pos = p.pos.xyz + vel * params.dt;

    particlesDst[index].pos = vec4<f32>(pos, p.pos.w);
    particlesDst[index].vel = vec4<f32>(vel, 0.0);
}
