struct Light {
    direction: vec3<f32>,
    strength: f32,
}

struct Lighting {
    u_lights: array<Light, 2>,
    u_ambient_light: f32,
}

struct FragmentOutput {
    @location(0) f_color: vec4<f32>,
}

@group(0) @binding(1) 
var<uniform> global: Lighting;
var<private> v_color_1: vec3<f32>;
var<private> v_normal_1: vec3<f32>;
var<private> f_color: vec4<f32>;

fn calc_darkness(normal: vec3<f32>, direction: vec3<f32>, strength: f32) -> f32 {
    var normal_1: vec3<f32>;
    var direction_1: vec3<f32>;
    var strength_1: f32;
    var brightness: f32;

    normal_1 = normal;
    direction_1 = direction;
    strength_1 = strength;
    let _e15 = normal_1;
    let _e16 = direction_1;
    let _e19 = strength_1;
    let _e25 = normal_1;
    let _e26 = direction_1;
    let _e29 = strength_1;
    brightness = clamp((-(dot(_e25, _e26)) * _e29), 0.0, 1.0);
    let _e36 = brightness;
    return (1.0 - _e36);
}

fn main_1() {
    var normal_2: vec3<f32>;
    var darkness: f32;
    var i: i32;
    var brightness_1: f32;

    let _e8 = v_normal_1;
    normal_2 = normalize(_e8);
    let _e12 = global.u_ambient_light;
    darkness = (1.0 - _e12);
    i = 0;
    loop {
        let _e17 = i;
        if !((_e17 < 2)) {
            break;
        }
        {
            let _e24 = darkness;
            let _e26 = i;
            let _e28 = global.u_lights[_e26];
            let _e30 = i;
            let _e32 = global.u_lights[_e30];
            let _e34 = normal_2;
            let _e35 = i;
            let _e37 = global.u_lights[_e35];
            let _e39 = i;
            let _e41 = global.u_lights[_e39];
            let _e43 = calc_darkness(_e34, _e37.direction, _e41.strength);
            darkness = (_e24 * _e43);
        }
        continuing {
            let _e21 = i;
            i = (_e21 + 1);
        }
    }
    let _e46 = darkness;
    brightness_1 = (1.0 - _e46);
    let _e49 = v_color_1;
    let _e50 = brightness_1;
    let _e51 = (_e49 * _e50);
    f_color = vec4<f32>(_e51.x, _e51.y, _e51.z, 1.0);
    return;
}

@fragment 
fn main(@location(0) v_color: vec3<f32>, @location(1) v_normal: vec3<f32>) -> FragmentOutput {
    v_color_1 = v_color;
    v_normal_1 = v_normal;
    main_1();
    let _e15 = f_color;
    return FragmentOutput(_e15);
}
