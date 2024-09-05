// uniform vec3 cameraPosition;

uniform vec3 sunPosition;
uniform float time;
uniform vec3 scatter_color;
uniform float scatter_factor;
uniform vec3 water_color;

varying vec3 normal_v;
varying vec3 vViewPosition;
varying float depth;

void main() {
  vec3 normal = normalize(normal_v);
  // vec3 normal = vec3(0.0, 0.0, 1.0);

  vec3 view_direction = normalize(cameraPosition - vViewPosition);
  // vec3 view_direction = vec3(0.0, 0.0, 1.0);

  vec3 reflected = normalize(reflect(-sunPosition, normal));
  float reflect_intensity = dot(view_direction, reflected);
  reflect_intensity = clamp(reflect_intensity, 0.0, 1.0);
  reflect_intensity = pow(reflect_intensity, 100.0);

  float diffusion_intensity = dot(normal, sunPosition);

  // Calculate subsurface scattering
  float scatter_intensity = exp(-scatter_factor * (1.0 - abs(diffusion_intensity)));
  vec3 scatter_effect = scatter_color * scatter_intensity;

  diffusion_intensity = clamp(diffusion_intensity, 0.0, 1.0);

  float intensity = 0.2 + 0.8 * diffusion_intensity + 1.0 * reflect_intensity;

  vec3 color = water_color * intensity + scatter_effect;
  //add fog effect
  color = mix(color, vec3(0.6, 0.6, 0.6), depth / 2000.0);
  gl_FragColor = vec4(color, 0.95);

}