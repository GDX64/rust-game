import * as THREE from "three";
import explosionURL from "../assets/explosion2.ogg";
// import explosionURL from "../assets/explosion.mp3";
import { awaitTime } from "../utils/promiseUtils";

const TOO_FAR = 2000;
export class ExplosionAudioManager {
  private audioBuffer: AudioBuffer | null = null;
  audioGroup = new THREE.Group();
  private freeAudioSet = new Set<THREE.PositionalAudio>();
  private soundsToPlay: THREE.Vector3[] = [];
  constructor(private listener: THREE.AudioListener) {
    this.freeAudioSet = new Set(
      [...Array(30)].map(() => {
        return new THREE.PositionalAudio(this.listener);
      })
    );

    this.freeAudioSet.forEach((audio) => {
      this.audioGroup.add(audio);
    });

    this.loadAudioBuffer();
  }

  playAt(position: THREE.Vector3) {
    if (!this.soundsToPlay.length) {
      setTimeout(() => this.groupSounds(), 100);
    }
    const distance = this.listener
      .getWorldPosition(new THREE.Vector3())
      .distanceTo(position);
    if (distance > TOO_FAR) {
      return;
    }
    this.soundsToPlay.push(position);
    // this.playForReal(position, 1);
  }

  private groupSounds() {
    if (!this.soundsToPlay.length) {
      return;
    }
    const positions = this.soundsToPlay;
    this.soundsToPlay = [];
    const averagePosition = new THREE.Vector3(0, 0, 0);
    for (const position of positions) {
      averagePosition.add(position);
    }
    averagePosition.divideScalar(positions.length);
    this.playForReal(averagePosition, positions.length);
  }

  private async playForReal(position: THREE.Vector3, numberOfSounds: number) {
    const audio = this.freeAudioSet.values().next().value;
    if (!audio || !this.audioBuffer) {
      return;
    }
    audio.position.copy(position);
    audio.setBuffer(this.audioBuffer);
    const MAX_VOLUME = 10;
    audio.setVolume(Math.min(0.1 * numberOfSounds, MAX_VOLUME));
    audio.play(0.1 * Math.random());
    audio.setRefDistance(20);

    audio.loop = false;
    this.freeAudioSet.delete(audio);
    await awaitTime(2000);
    audio.stop();
    this.freeAudioSet.add(audio);
  }

  async loadAudioBuffer() {
    const audioLoader = new THREE.AudioLoader();
    document.addEventListener(
      "click",
      () => {
        this.listener.context.resume();
      },
      { once: true }
    );
    await new Promise<void>((resolve) => {
      this.listener.context.addEventListener("statechange", () => {
        if (this.listener.context.state === "running") {
          resolve();
        }
      });
    });
    audioLoader.load(explosionURL, (buffer) => {
      this.audioBuffer = buffer;
    });
  }
}
