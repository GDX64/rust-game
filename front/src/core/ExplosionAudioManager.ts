import * as THREE from "three";
import explosionURL from "../assets/explosion2.ogg";
// import explosionURL from "../assets/explosion.mp3";
import { awaitTime } from "../utils/promiseUtils";

const TOO_FAR = 1000;
export class ExplosionAudioManager {
  private audioBuffer: AudioBuffer | null = null;
  audioGroup = new THREE.Group();
  private freeAudioSet = new Set<THREE.PositionalAudio>();
  private soundsToPlay: THREE.Vector3[] = [];
  constructor(private listener: THREE.AudioListener) {
    this.freeAudioSet = new Set(
      [...Array(100)].map(() => {
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
      setTimeout(() => this.groupSounds(), 16);
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
    debugger;
    const averagePosition = new THREE.Vector3(0, 0, 0);
    for (const position of positions) {
      averagePosition.add(position);
    }
    averagePosition.divideScalar(positions.length);
    console.log("Playing audio at", averagePosition);
    this.playForReal(averagePosition, positions.length);
  }

  private async playForReal(position: THREE.Vector3, numberOfSounds: number) {
    const audio = this.freeAudioSet.values().next().value;
    if (!audio || !this.audioBuffer) {
      return;
    }
    audio.position.copy(position);
    console.log("Playing audio at", audio.parent?.position);
    audio.setBuffer(this.audioBuffer);
    const MAX_VOLUME = 10;
    audio.setVolume(Math.min(0.5 * numberOfSounds, MAX_VOLUME));
    audio.play();
    audio.setRefDistance(20);

    console.log(audio.duration);
    audio.loop = false;
    this.freeAudioSet.delete(audio);
    await awaitTime(2000);
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
