// uniform vec3 cameraPosition;

varying vec3 normal_v;
varying vec3 vViewPosition;

void main() {
  vec3 sun_light = normalize(vec3(1.0, 1.0, 1.0));
  vec3 normal = normalize(normal_v);

  vec3 view_direction = normalize(cameraPosition - vViewPosition);
  // vec3 view_direction = vec3(0.0, 0.0, 1.0);

  vec3 reflected = reflect(sun_light, normal);
  float reflect_intensity = dot(reflected, view_direction);
  reflect_intensity = clamp(pow(reflect_intensity, 10.0), 0.0, 1.0);

  float diffusion_intensity = dot(normal, sun_light);
  diffusion_intensity = clamp(diffusion_intensity, 0.0, 1.0);

  float intensity = 0.2 + 0.8 * diffusion_intensity + 0.5 * reflect_intensity;

  gl_FragColor = vec4(0.23, 0.84, 0.92, 1.0) * intensity;
}