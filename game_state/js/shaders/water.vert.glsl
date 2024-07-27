uniform float time;

varying vec3 normal_v;
varying vec3 vViewPosition;

float z_vec(vec3 pos) {
  float acc = sin(pos.x * 0.1 + time * 2.0);
  // return acc;
  return 0.0;
}

vec3 grad_x(vec3 pos) {
  float delta_x = 0.01;
  vec3 delta = vec3(delta_x, 0.0, 0.0);
  float delta_z = z_vec(pos + delta) - z_vec(pos - delta);
  return vec3(delta_x, 0.0, delta_z / 2.0);
}

vec3 grad_y(vec3 pos) {
  float delta_y = 0.01;
  vec3 delta = vec3(0.0, delta_y, 0.0);
  float delta_z = z_vec(pos + delta) - z_vec(pos - delta);
  return vec3(0.0, delta_y, delta_z / 2.0);
}

vec3 normal_vec(vec3 pos) {
  vec3 x = grad_x(pos);
  vec3 y = grad_y(pos);
  return normalize(cross(x, y));
}

void main() {
  vec3 pos = position;
  vec4 my_pos = modelMatrix * vec4(pos, 1.0);
  my_pos.z = z_vec(my_pos.xyz) * 1.0;
  vViewPosition = my_pos.xyz;
  gl_Position = projectionMatrix * viewMatrix * my_pos;
  normal_v = normal_vec(pos);
  // normal_v = vec3(0.0, 0.0, 1.0);
}
