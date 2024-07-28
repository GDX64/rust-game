uniform float progress;
uniform vec3 color;

void main() {
  float opacity = 1.0 - progress;
  opacity = pow(opacity, 2.0);
  vec3 final_color = mix(color, vec3(0.0), progress);
  gl_FragColor = vec4(final_color, opacity);
}