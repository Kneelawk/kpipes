#version 450

#define NUM_LIGHTS 2

struct Light {
    vec3 direction;
    float strength;
};

layout(set = 0, binding = 1) uniform Lighting {
    Light u_lights[NUM_LIGHTS];
    float u_ambient_light;
};

layout(location = 0) in vec3 v_color;
layout(location = 1) in vec3 v_normal;

layout(location = 0) out vec4 f_color;

float calc_darkness(vec3 normal, vec3 direction, float strength) {
    float brightness = clamp(-dot(normal, direction) * strength, 0.0, 1.0);

    return 1.0 - brightness;
}

void main() {
    vec3 normal = normalize(v_normal);

    float darkness = 1.0 - u_ambient_light;

    for (int i = 0; i < NUM_LIGHTS; i++) {
        darkness *= calc_darkness(normal, u_lights[i].direction, u_lights[i].strength);
    }

    float brightness = 1.0 - darkness;

    f_color = vec4(v_color * brightness, 1.0);
}
