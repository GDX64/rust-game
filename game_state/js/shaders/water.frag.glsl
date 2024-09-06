// uniform vec3 cameraPosition;

uniform vec3 sunPosition;
uniform float time;
uniform vec3 scatter_color;
uniform float scatter_factor;
uniform vec3 water_color;
uniform sampler2D normal_map;
uniform float texture_scale;
uniform float z_gain;

varying vec3 normal_v;
varying vec3 vViewPosition;
varying float depth;

vec3 get_displacement() {
  vec2 uv = vViewPosition.xy / texture_scale;
  float t = time * 0.003;
  vec2 uv_offset1 = uv + vec2(t, 0.0);
  vec2 uv_offset2 = uv + vec2(-t, 0.0);
  vec4 normal1 = texture2D(normal_map, uv_offset1);
  vec4 normal2 = texture2D(normal_map, uv_offset2);
  normal1.xy = normal1.xy * 2.0 - 1.0;
  normal2.xy = normal2.xy * 2.0 - 1.0;
  vec3 normal = normal1.xyz * 0.5 + normal2.xyz * 0.5;
  normal.z = normal.z * z_gain;
  return normalize(normal);
}

void main() {
  vec3 normal_displace = get_displacement();
  vec3 normal = normalize(normal_v + normal_displace.xyz);

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