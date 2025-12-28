// Vertex Shader
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

    let world_pos = instance.position + vec3<f32>(uv * instance.radius, 0.0);
    
    var out: VertexOutput;
    out.clip_position = view.view_proj * vec4<f32>(world_pos, 1.0);
    out.color = instance.color;
    out.uv = uv;
    return out;
}

// Fragment Shader
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // 1. Calculate signed distance field (sphere)
    let uv = in.uv; // -1 to 1
    let r2 = dot(uv, uv);
    if (r2 > 1.0) {
        discard;
    }
    
    // 2. Calculate Normal (Hemisphere)
    let z = sqrt(1.0 - r2);
    let normal = vec3<f32>(uv.x, uv.y, z);
    
    // 3. Lighting
    let light_dir = normalize(vec3<f32>(0.5, 0.8, 1.0));
    
    // Diffuse
    let diff = max(dot(normal, light_dir), 0.0);
    
    // Specular (Blinn-Phong)
    let view_dir = vec3<f32>(0.0, 0.0, 1.0); // Ortho: view is always +Z
    let half_dir = normalize(light_dir + view_dir);
    let spec = pow(max(dot(normal, half_dir), 0.0), 32.0);
    
    // Ambient
    let ambient = 0.2;
    
    // Fresnel Reflection (Mock)
    let fresnel = pow(1.0 - max(dot(normal, view_dir), 0.0), 2.0);
    
    // Composition
    // Fluid Color (Blue-ish)
    let fluid_color = in.color; // Using instance color
    
    // Combine
    let val = (vec3(ambient) + vec3(diff)) * fluid_color + vec3(spec * 0.8) + vec3(fresnel * 0.3);
    
    // Soft Edge Antialiasing
    let alpha = smoothstep(1.0, 0.85, sqrt(r2));
    
    return vec4<f32>(val, alpha);
}
