// Refraction Shader (Screen Space)

struct ViewUniform {
    view_proj: mat4x4<f32>,
};
@group(0) @binding(0) var<uniform> view: ViewUniform;

struct InstanceInput {
    @location(0) position: vec3<f32>,
    @location(1) radius: f32,
    @location(2) color: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) world_pos: vec3<f32>,
};

@vertex
fn vs_main(
    @builtin(vertex_index) in_vertex_index: u32,
    instance: InstanceInput,
) -> VertexOutput {
    var uvs = array<vec2<f32>, 4>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>( 1.0, -1.0),
        vec2<f32>(-1.0,  1.0),
        vec2<f32>( 1.0,  1.0)
    );
    let uv = uvs[in_vertex_index];

    let radius = instance.radius;
    let world_pos = instance.position + vec3<f32>(uv * radius, 0.0);
    
    var out: VertexOutput;
    out.clip_position = view.view_proj * vec4<f32>(world_pos, 1.0);
    out.color = vec3<f32>(0.2, 0.6, 1.0); // Hardcoded Blue Tint
    out.uv = uv;
    out.world_pos = world_pos;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // 1. Calculate sphere shape (SDF)
    let uv = in.uv; // -1 to 1
    let r2 = dot(uv, uv);
    if (r2 > 1.0) {
        discard;
    }
    
    // 2. Normal (Hemisphere)
    let z = sqrt(1.0 - r2);
    let normal = vec3<f32>(uv.x, uv.y, z);
    
    // 3. Grid Background (for refraction)
    let grid_size = 50.0;
    // Distort world pos by normal for refraction effect
    let refracted_pos = in.world_pos - normal * 10.0; 
    
    let grid_x = step(0.9, fract(refracted_pos.x / grid_size));
    let grid_y = step(0.9, fract(refracted_pos.y / grid_size));
    let grid = max(grid_x, grid_y);
    let bg_color = mix(vec3<f32>(0.1, 0.1, 0.15), vec3<f32>(0.3, 0.3, 0.35), grid);

    // 4. Lighting (Blinn-Phong)
    let light_dir = normalize(vec3<f32>(0.5, 0.8, 1.0));
    let view_dir = vec3<f32>(0.0, 0.0, 1.0);
    let half_dir = normalize(light_dir + view_dir);
    
    let spec = pow(max(dot(normal, half_dir), 0.0), 64.0);
    let fresnel = pow(1.0 - max(dot(normal, view_dir), 0.0), 3.0);
    
    // 5. Composition
    // Fluid absorbs light (Beers law-ish), so we tint the background
    let tint = in.color;
    
    // Mix Refracted Background with Surface Reflection
    let col = mix(bg_color * tint, vec3<f32>(1.0), spec + fresnel * 0.5);
    
    // Soft Edge AA
    let alpha = smoothstep(1.0, 0.85, sqrt(r2));
    
    return vec4<f32>(col, alpha);
}
