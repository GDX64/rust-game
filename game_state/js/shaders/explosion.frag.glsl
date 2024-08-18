uniform float progress;
uniform vec3 color;

const vec3 white = vec3(0.5, 0.5, 0.5);
uniform sampler2D diffuseTexture;

varying vec2 vAngle;

void main() {
  float opacity = 1.0 - progress;
  opacity = pow(opacity, 6.0);
  vec2 coord = gl_PointCoord - 0.5;
  coord = mat2(vAngle, -vAngle.y, vAngle.x) * coord + 0.5;
  // gl_FragColor = vec4(final_color, opacity);
  vec4 texture_color = texture2D(diffuseTexture, coord);
  vec3 final_color = mix(texture_color.xyz, white, progress);
  gl_FragColor = vec4(final_color, texture_color.a * opacity);
}