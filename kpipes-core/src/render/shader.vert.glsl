#version 450

layout(set = 0, binding = 0) uniform Uniforms {
    mat4 u_vp_matrix;
};

layout(location = 0) in vec3 s_color;
layout(location = 1) in mat4 s_model;

layout(location = 5) in vec3 a_position;
layout(location = 6) in vec3 a_normal;

layout(location = 0) out vec3 v_color;
layout(location = 1) out vec3 v_normal;

void main() {
    gl_Position = u_vp_matrix * s_model * vec4(a_position, 1.0);
    v_color = s_color;
    v_normal = normalize((s_model * vec4(a_normal, 0.0)).xyz);
}
