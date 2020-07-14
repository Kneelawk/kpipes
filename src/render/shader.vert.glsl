#version 450

layout(set = 0, binding = 0) uniform Uniforms {
    mat4 u_vp_matrix;
};

layout(location = 0) in vec3 a_position;
layout(location = 1) in vec3 a_color;

layout(location = 0) out vec3 v_color;

void main() {
    gl_Position = u_vp_matrix * vec4(a_position, 1.0);
    v_color = a_color;
}
