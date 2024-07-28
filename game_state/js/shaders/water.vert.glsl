const int DIR = 8;

uniform float time;
uniform vec2 directions[DIR];
uniform float amplitude;

varying vec3 normal_v;
varying vec3 vViewPosition;

void wave(vec3 pos, out float wave, out vec3 normal) {
  float z_acc = 0.0;
  vec2 derivative_acc = vec2(0.0, 0.0);
  for(int i = 0; i < DIR; i++) {
    float harmonic = float(i + 1);
    float angle = dot(directions[i], pos.xy) + time;

    z_acc += sin(angle) / harmonic / harmonic;
    derivative_acc += directions[i] * cos(angle) / harmonic / harmonic;
  }
  wave = z_acc * amplitude;
  derivative_acc *= amplitude;
  vec3 grad_x = vec3(1.0, 0.0, derivative_acc.x);
  vec3 grad_y = vec3(0.0, 1.0, derivative_acc.y);
  normal = normalize(cross(grad_x, grad_y));
}

void main() {
  vec4 pos = vec4(position, 1.0);
  vec3 normal_calc;
  wave(pos.xyz, pos.z, normal_calc);
  vViewPosition = (modelMatrix * pos).xyz;
  gl_Position = projectionMatrix * modelViewMatrix * pos;
  normal_v = normal_calc;
  // normal_v = vec3(0.0, 0.0, 1.0);
}
