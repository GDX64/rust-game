uniform float progress;
uniform vec3 color;

const vec3 white = vec3(0.5, 0.5, 0.5);
uniform sampler2D diffuseTexture;

void main() {
  float opacity = 1.0 - progress;
  opacity = pow(opacity, 6.0);
  // gl_FragColor = vec4(final_color, opacity);
  vec4 texture_color = texture2D(diffuseTexture, gl_PointCoord);
  vec3 final_color = mix(texture_color.xyz, white, progress);
  gl_FragColor = vec4(final_color, texture_color.a * opacity);
}