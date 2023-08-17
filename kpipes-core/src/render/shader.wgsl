// Combined WGSL shader

// Consts

const NUM_LIGHTS: u32 = 2u;

// Structs

// Matrix Uniform
struct Uniforms {
    u_vp_matrix: mat4x4<f32>,
}

// Light
struct Light {
    direction: vec3<f32>,
    strength: f32,
}

// Lighting Uniform
struct Lighting {
    u_lights: array<Light, NUM_LIGHTS>,
    u_ambient_light: f32,
    _padding1: u32,
    _padding2: u32,
    _padding3: u32,
}

// Vertex Attributes
struct VertexAttributes {
    @location(0)
    s_color: vec3<f32>,
    @location(1)
    s_model_0: vec4<f32>,
    @location(2)
    s_model_1: vec4<f32>,
    @location(3)
    s_model_2: vec4<f32>,
    @location(4)
    s_model_3: vec4<f32>,
    @location(5)
    a_position: vec3<f32>,
    @location(6)
    a_normal: vec3<f32>,
}

// Fragment Attributes
struct FragmentAttributes {
    @builtin(position)
    position: vec4<f32>,
    @location(0)
    v_color: vec3<f32>,
    @location(1)
    v_normal: vec3<f32>,
}

// Fragment Output
struct FragmentOutput {
    @location(0)
    f_color: vec4<f32>,
}

// Uniforms

@group(0)
@binding(0)
var<uniform> uniforms: Uniforms;
@group(0)
@binding(1)
var<uniform> lighting: Lighting;

// Vertex Shader

@vertex
fn vert_main(vertex: VertexAttributes) -> FragmentAttributes {
    let s_model = mat4x4f(vertex.s_model_0, vertex.s_model_1, vertex.s_model_2, vertex.s_model_3);

    let position = uniforms.u_vp_matrix * s_model * vec4f(vertex.a_position, 1.0);
    let normal = normalize((s_model * vec4f(vertex.a_normal, 0.0)).xyz);

    return FragmentAttributes(position, vertex.s_color, normal);
}

// Fragment Shader

fn calc_darkness(normal: vec3<f32>, direction: vec3<f32>, strength: f32) -> f32 {
    let brightness = clamp(-dot(normal, direction) * strength, 0.0, 1.0);

    return 1.0 - brightness;
}

@fragment
fn frag_main(fragment: FragmentAttributes) -> FragmentOutput {
    let normal = normalize(fragment.v_normal);

    var darkness = 1.0 - lighting.u_ambient_light;

    for (var i = 0u; i < NUM_LIGHTS; i++) {
        let light = lighting.u_lights[i];
        darkness *= calc_darkness(normal, light.direction, light.strength);
    }

    let brightness = 1.0 - darkness;
    let color = vec4f(fragment.v_color * brightness, 1.0);

    return FragmentOutput(color);
}
