uniform float time;
uniform float progress;

attribute vec3 speed;

void main() {
  vec3 pos = speed * time;
  gl_Position = projectionMatrix * modelViewMatrix * vec4(pos, 1.0);
  gl_PointSize = ((1.0 - progress) * 5.0) / gl_Position.z / gl_Position.z;
  // normal_v = vec3(0.0, 0.0, 1.0);
}
