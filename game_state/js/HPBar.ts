import * as THREE from "three";

const RED = new THREE.Color(1, 0, 0);
const GREEN = new THREE.Color(0, 1, 0);
const YELLOW = new THREE.Color(1, 1, 0);

function getHPColor(hp: number) {
  if (hp > 50) {
    return GREEN;
  } else if (hp > 20) {
    return YELLOW;
  } else {
    return RED;
  }
}

export class HPBar {
  material;
  hpBar;
  reausableMatrix = new THREE.Matrix4();
  reausableVector = new THREE.Vector3();
  constructor() {
    const geometry = new THREE.PlaneGeometry(10, 1);
    const material = new THREE.ShaderMaterial({
      side: THREE.DoubleSide,
      fragmentShader,
      vertexShader,
      colorWrite: true,
    });
    const hpBar = new THREE.InstancedMesh(geometry, material, 500);
    //I need to do this so that threejs initializes the instanceColors
    hpBar.setColorAt(0, GREEN);
    hpBar.frustumCulled = false;
    this.hpBar = hpBar;
    this.material = material;
  }

  addToScene(scene: THREE.Scene) {
    scene.add(this.hpBar);
  }

  setInstancesCount(count: number) {
    this.hpBar.count = count;
  }

  updateBar(i: number, shipMatrix: THREE.Matrix4, hp: number) {
    const x = shipMatrix.elements[12];
    const y = shipMatrix.elements[13];
    const z = shipMatrix.elements[14] + 10;
    this.reausableMatrix.makeScale(hp / 100, 1, 1);
    this.reausableMatrix.setPosition(x, y, z);
    this.hpBar.setMatrixAt(i, this.reausableMatrix);
    this.hpBar.setColorAt(i, getHPColor(hp));
    this.hpBar.instanceMatrix.needsUpdate = true;
    this.hpBar.instanceColor!.needsUpdate = true;
    this.hpBar.material.needsUpdate = true;
  }
}

const fragmentShader = `
      varying vec3 passColor;

      void main(){
        gl_FragColor = vec4(passColor, 1.0);
      }
      `;

const vertexShader = /*glsl*/ `

varying vec3 passColor;

void main() {

	vec4 mvPosition = modelViewMatrix * instanceMatrix * vec4( 0.0, 0.0, 0.0, 1.0 );
  vec4 pos4 = vec4( position, 1.0 );

  vec2 scale = vec2(1.0, 1.0);
	scale.x = length( vec3( instanceMatrix[ 0 ].x, instanceMatrix[ 0 ].y, instanceMatrix[ 0 ].z ) );
	scale.y = length( vec3( instanceMatrix[ 1 ].x, instanceMatrix[ 1 ].y, instanceMatrix[ 1 ].z ) );

  pos4.xy = pos4.xy * scale;
  
  pos4+=mvPosition;

	gl_Position = projectionMatrix * pos4;
  passColor = instanceColor;

}
      `;
