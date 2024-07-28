uniform float progress;

void main() {
  float opacity = 1.0 - progress;
  opacity = pow(opacity, 2.0);
  float color = 1.0 - progress;
  color = pow(color, 4.0);
  gl_FragColor = vec4(color, color, color, opacity);
}