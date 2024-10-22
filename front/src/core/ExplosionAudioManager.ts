import * as THREE from "three";
import shotURL from "../assets/explosion2.ogg";
import explosionURL from "../assets/explosion.ogg";
import { awaitTime } from "../utils/promiseUtils";
import { ExplosionKind } from "rust";

const MAX_SIMULTANEOUS_SOUNDS = 30;

export class ExplosionAudioManager {
  static refdistance = 100;

  private explosionBuffer: AudioBuffer | null = null;
  private shotBuffer: AudioBuffer | null = null;
  audioGroup = new THREE.Group();
  private freeAudioSet = new Set<THREE.PositionalAudio>();
  private soundsToPlay: { position: THREE.Vector3; kind: ExplosionKind }[] = [];
  constructor(private listener: THREE.AudioListener) {
    this.freeAudioSet = new Set(
      [...Array(MAX_SIMULTANEOUS_SOUNDS)].map(() => {
        const audio = new THREE.PositionalAudio(this.listener);

        // audio.panner.distanceModel = "linear";
        // audio.panner.rolloffFactor = 1;
        // audio.panner.refDistance = 1;

        return audio;
      })
    );

    this.freeAudioSet.forEach((audio) => {
      this.audioGroup.add(audio);
    });

    this.loadAudioBuffer();
  }

  playAt(position: THREE.Vector3, kind: ExplosionKind) {
    if (!this.soundsToPlay.length) {
      setTimeout(() => this.groupSounds(), 100);
    }
    this.soundsToPlay.push({ position, kind });
  }

  private groupSounds() {
    if (!this.soundsToPlay.length) {
      return;
    }

    const positions = groupBy(this.soundsToPlay, (sound) => sound.kind);
    for (const [kind, values] of positions) {
      this.soundsToPlay = [];
      const averagePosition = new THREE.Vector3(0, 0, 0);
      for (const { position } of values) {
        averagePosition.add(position);
      }
      averagePosition.divideScalar(values.length);
      this.playForReal(averagePosition, values.length, kind);
    }
  }

  private async playForReal(
    position: THREE.Vector3,
    numberOfSounds: number,
    kind: ExplosionKind
  ) {
    const audio = this.freeAudioSet.values().next().value;
    if (!audio || !this.explosionBuffer || !this.shotBuffer) {
      return;
    }
    audio.position.copy(position);
    if (kind === ExplosionKind.Shot) {
      audio.setBuffer(this.shotBuffer);
    } else {
      audio.setBuffer(this.explosionBuffer);
    }
    const MAX_VOLUME = 10;
    audio.setVolume(Math.min(0.1 * numberOfSounds, MAX_VOLUME));
    audio.play(0.1 * Math.random());
    audio.setRefDistance(ExplosionAudioManager.refdistance);

    audio.loop = false;
    this.freeAudioSet.delete(audio);
    const time = kind === ExplosionKind.Shot ? 2000 : 3000;
    await awaitTime(time);
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
    audioLoader.load(shotURL, (buffer) => {
      this.shotBuffer = buffer;
    });
    audioLoader.load(explosionURL, (buffer) => {
      this.explosionBuffer = buffer;
    });
  }
}

function groupBy<T, K>(array: T[], key: (item: T) => K) {
  const groups = new Map<K, T[]>();
  for (const item of array) {
    const groupKey = key(item);
    if (!groups.has(groupKey)) {
      groups.set(groupKey, []);
    }
    groups.get(groupKey)!.push(item);
  }
  return groups;
}
