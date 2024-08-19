uniform float time;
uniform float progress;

uniform float pointMultiplier;
attribute vec3 speed;

varying vec2 vAngle;

void main() {
  vec3 pos = speed * sqrt(time);
  gl_Position = projectionMatrix * modelViewMatrix * vec4(pos, 1.0);
  gl_PointSize = pointMultiplier * (5000.0) / gl_Position.w;
  gl_PointSize = floor(gl_PointSize);
  float angle = time;
  vAngle = vec2(cos(angle), sin(angle));
  // normal_v = vec3(0.0, 0.0, 1.0);
}
