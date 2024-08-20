uniform float progress;
uniform vec3 color;

uniform sampler2D diffuseTexture;

varying vec2 vAngle;
varying float vDistanceProgress;

void main() {
  vec2 coord = gl_PointCoord - 0.5;
  coord = mat2(vAngle, -vAngle.y, vAngle.x) * coord + 0.5;
  // gl_FragColor = vec4(final_color, opacity);
  vec4 texture_color = texture2D(diffuseTexture, coord);

  float smoke = smoothstep(0.25, 0.3, vDistanceProgress);
  smoke = max(smoothstep(0.3, 0.5, progress), smoke);
  float opacity = 1.0 - progress;
  opacity = pow(opacity, 2.0);
  vec3 final_color = texture_color.xyz * texture_color.a * opacity * (1.0 - smoke);

  //pra ser aditivo, o alpha tem que ser 0
  gl_FragColor = vec4(final_color, texture_color.a * opacity * smoke);
}