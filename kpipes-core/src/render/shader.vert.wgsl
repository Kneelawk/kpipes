struct Uniforms {
  /* @offset(0) */
  u_vp_matrix : mat4x4<f32>,
}

@group(0) @binding(0) var<uniform> x_19 : Uniforms;

var<private> s_model : mat4x4<f32>;

var<private> a_position : vec3<f32>;

var<private> v_color : vec3<f32>;

var<private> s_color : vec3<f32>;

var<private> v_normal : vec3<f32>;

var<private> a_normal : vec3<f32>;

var<private> gl_Position : vec4<f32>;

fn main_1() {
  let x_22 : mat4x4<f32> = x_19.u_vp_matrix;
  let x_25 : mat4x4<f32> = s_model;
  let x_30 : vec3<f32> = a_position;
  gl_Position = ((x_22 * x_25) * vec4<f32>(x_30.x, x_30.y, x_30.z, 1.0f));
  let x_42 : vec3<f32> = s_color;
  v_color = x_42;
  let x_44 : mat4x4<f32> = s_model;
  let x_46 : vec3<f32> = a_normal;
  let x_52 : vec4<f32> = (x_44 * vec4<f32>(x_46.x, x_46.y, x_46.z, 0.0f));
  v_normal = normalize(vec3<f32>(x_52.x, x_52.y, x_52.z));
  return;
}

struct main_out {
  @builtin(position)
  gl_Position : vec4<f32>,
  @location(0)
  v_color_1 : vec3<f32>,
  @location(1)
  v_normal_1 : vec3<f32>,
}

@vertex
fn main(@location(1) s_model_param : vec4<f32>, @location(2) s_model_param_1 : vec4<f32>, @location(3) s_model_param_2 : vec4<f32>, @location(4) s_model_param_3 : vec4<f32>, @location(5) a_position_param : vec3<f32>, @location(0) s_color_param : vec3<f32>, @location(6) a_normal_param : vec3<f32>) -> main_out {
  s_model[0i] = s_model_param;
  s_model[1i] = s_model_param_1;
  s_model[2i] = s_model_param_2;
  s_model[3i] = s_model_param_3;
  a_position = a_position_param;
  s_color = s_color_param;
  a_normal = a_normal_param;
  main_1();
  return main_out(gl_Position, v_color, v_normal);
}
