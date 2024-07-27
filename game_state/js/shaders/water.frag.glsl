// uniform vec3 cameraPosition;

uniform vec3 sunPosition;

varying vec3 normal_v;
varying vec3 vViewPosition;

void main() {
  vec3 sun_light = sunPosition;
  vec3 normal = normalize(normal_v);
  // vec3 normal = vec3(0.0, 0.0, 1.0);

  vec3 view_direction = normalize(cameraPosition - vViewPosition);
  // vec3 view_direction = vec3(0.0, 0.0, 1.0);

  vec3 reflected = normalize(reflect(-sun_light, normal));
  float reflect_intensity = dot(view_direction, reflected);
  reflect_intensity = clamp(reflect_intensity, 0.0, 1.0);
  reflect_intensity = pow(reflect_intensity, 50.0);

  float diffusion_intensity = dot(normal, sun_light);
  diffusion_intensity = clamp(diffusion_intensity, 0.0, 1.0);

  float intensity = 0.1 + 0.8 * diffusion_intensity + 0.5 * reflect_intensity;

  vec3 color = vec3(0.2, 0.42, 0.64) * intensity;
  gl_FragColor = vec4(color, 1.0);
}