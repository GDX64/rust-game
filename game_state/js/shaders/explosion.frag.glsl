uniform float progress;

void main() {
  float alpha = 1.0 - progress;
  alpha = pow(alpha, 2.0);
  gl_FragColor = vec4(1.0, 1.0, 1.0, alpha);
}