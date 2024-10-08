import * as THREE from "three";

export function drawImageShader(
  texture: THREE.Texture,
  width: number,
  height: number
) {
  const material = new THREE.ShaderMaterial({
    fragmentShader: /*glsl*/ `
      varying vec2 vUv;
      uniform sampler2D canvasTexture;
      void main() {
        vec4 tex = texture2D(canvasTexture, vUv);
        tex.a = 0.5;
        //tex.x = vUv.x;
        //tex.y = vUv.y;
        gl_FragColor = tex;
        //gl_FragColor = vec4(1.0, 0.0, 0.0, 1.0);
      }
      `,
    vertexShader: /*glsl*/ `
      varying vec2 vUv;
      uniform float width;
      uniform float height;
      uniform float screenWidth;
      uniform float screenHeight;
      
      void main() {
        vUv = uv;
        vec4 pos = vec4(position, 1.0);
        float boxWidth = 2.0*width / (screenWidth) ;
        float boxHeight = 2.0*height / (screenHeight);
        pos.x = (pos.x + 0.5)*boxWidth + 1.0 - boxWidth;
        pos.y = (pos.y - 0.5)*boxHeight - 1.0 + boxHeight;
        pos.z = .5;
        gl_Position = pos;
      }
      `,
    uniforms: {
      canvasTexture: { value: texture },
      width: { value: width },
      height: { value: height },
      screenWidth: { value: window.innerWidth },
      screenHeight: { value: window.innerHeight },
    },
    depthTest: false,
    depthWrite: false,
    transparent: true,
  });
  return material;
}
