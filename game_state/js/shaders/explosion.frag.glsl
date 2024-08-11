uniform float progress;
uniform vec3 color;

void main() {
  float opacity = 1.0 - progress;
  opacity = pow(opacity, 3.0);
  vec3 final_color = mix(color, vec3(0.0, 0.0, 0.0), pow(progress, 1.0 / 10.0));
  gl_FragColor = vec4(final_color, opacity);
}