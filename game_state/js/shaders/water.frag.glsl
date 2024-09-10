// uniform vec3 cameraPosition;

uniform vec3 sunPosition;
uniform float time;
uniform vec3 scatter_color;
uniform float scatter_factor;
uniform vec3 water_color;
uniform sampler2D normal_map;
uniform sampler2D height_texture;
uniform float texture_scale;
uniform float z_gain;

const vec3 SHALLOW_COLOR = vec3(0.0, 0.9, 1.0);
const float FOAM_TEXTURE_SCALE = 0.003;
const vec3 WHITE = vec3(1.0, 1.0, 1.0);

varying vec3 normal_v;
varying vec3 vViewPosition;
varying vec2 vUv;
varying float depth;

vec3 get_displacement() {
  vec2 uv = vViewPosition.xy / texture_scale;
  float t = time / 10.0;
  vec2 uv_offset1 = uv + vec2(t / 17.0, t / 29.0);
  vec2 uv_offset2 = uv + vec2(-t / 19.0, t / 32.0);
  vec2 uv_offset3 = uv + vec2(t / 101.0, -t / 113.0);
  vec2 uv_offset4 = uv + vec2(-t / 109.0, t / 97.0);
  vec4 normal1 = texture2D(normal_map, uv_offset1);
  vec4 normal2 = texture2D(normal_map, uv_offset2);
  vec4 normal3 = texture2D(normal_map, uv_offset3);
  vec4 normal4 = texture2D(normal_map, uv_offset4);
  vec3 normal = normal1.xyz + normal2.xyz + normal3.xyz + normal4.xyz;
  normal.z = normal.z * z_gain;
  return normalize(normal);
}

float get_foam() {
  vec2 uv = vViewPosition.xy * FOAM_TEXTURE_SCALE;
  float t = time * FOAM_TEXTURE_SCALE * 20.0;
  vec2 uv_offset1 = uv + vec2(t / 17.0, t / 29.0);
  vec2 uv_offset2 = uv + vec2(-t / 19.0, t / 31.0);
  float foam_noise1 = texture2D(normal_map, uv_offset1).r;
  float foam_noise2 = texture2D(normal_map, uv_offset2).g;
  float foam_noise = (foam_noise1 + foam_noise2) / 2.0;
  float coast_distance = texture2D(height_texture, vUv).g;
  float foam_influence = 1.0 - smoothstep(0.0, 0.015, coast_distance);
  return foam_influence * foam_noise;
}

vec3 get_water_height_color() {
  float height_value = texture2D(height_texture, vUv).r;
  return mix(water_color, SHALLOW_COLOR, height_value);
}

void main() {
  vec3 normal_displace = get_displacement();
  vec3 normal = normalize(normal_v + normal_displace.xyz);

  vec3 view_direction = normalize(cameraPosition - vViewPosition);
  // vec3 view_direction = vec3(0.0, 0.0, 1.0);

  vec3 reflected = normalize(reflect(-sunPosition, normal));
  float reflect_intensity = abs(dot(view_direction, reflected));
  reflect_intensity = clamp(reflect_intensity, 0.0, 1.0);
  reflect_intensity = pow(reflect_intensity, scatter_factor);

  float diffusion_intensity = dot(normal, sunPosition);

  // Calculate subsurface scattering
  // float scatter_intensity = exp(-scatter_factor * (1.0 - abs(diffusion_intensity)));

  diffusion_intensity = clamp(diffusion_intensity, 0.0, 1.0);

  float intensity = 0.2 + 0.8 * diffusion_intensity;
  vec3 height_color = get_water_height_color();

  vec3 color = height_color * intensity + scatter_color * reflect_intensity;
  //add fog effect
  color = mix(color, vec3(0.6, 0.6, 0.6), depth / 2000.0);
  color = mix(color, WHITE, get_foam());

  gl_FragColor = vec4(color, 0.9);

}