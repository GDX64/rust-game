import * as THREE from "three";
import explosionURL from "../assets/explosion.mp3";
import { awaitTime } from "../utils/promiseUtils";
export class ExplosionAudioManager {
  audioBuffer: AudioBuffer | null = null;
  audioGroup = new THREE.Group();
  freeAudioSet = new Set<THREE.PositionalAudio>();
  constructor(private listener: THREE.AudioListener) {
    this.freeAudioSet = new Set(
      [...Array(10)].map(() => {
        return new THREE.PositionalAudio(this.listener);
      })
    );

    this.freeAudioSet.forEach((audio) => {
      this.audioGroup.add(audio);
    });

    this.loadAudioBuffer();
  }

  async playAt(position: THREE.Vector3) {
    const audio = this.freeAudioSet.values().next().value;
    if (!audio || !this.audioBuffer) {
      return;
    }
    audio.position.copy(position);
    console.log("Playing audio at", audio.parent?.position);
    audio.setBuffer(this.audioBuffer);
    audio.setVolume(50);
    audio.play();

    console.log(audio.duration);
    audio.loop = false;
    this.freeAudioSet.delete(audio);
    await awaitTime(3000);
    audio.stop();
    console.log("Audio ended");
    this.freeAudioSet.add(audio);
  }

  async loadAudioBuffer() {
    const audioLoader = new THREE.AudioLoader();
    document.addEventListener(
      "click",
      () => {
        this.listener.context.resume();

        console.log("resumed", this.listener.context.state);
      },
      { once: true }
    );
    await new Promise<void>((resolve) => {
      this.listener.context.addEventListener("statechange", () => {
        console.log(this.listener.context.state);
        if (this.listener.context.state === "running") {
          resolve();
        }
      });
    });
    console.log("Audio context is running");
    audioLoader.load(explosionURL, (buffer) => {
      this.audioBuffer = buffer;
      console.log("Audio loaded", buffer);
    });
  }
}
