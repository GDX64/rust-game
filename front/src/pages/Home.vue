<template>
  <div class="h-full w-full flex items-end justify-center">
    <div
      class="absolute -z-1 w-full h-[30%] top-0 left-0 bg-gradient-to-b from-prime-100 to-white opacity-80"
    ></div>
    <div class="absolute -z-1 bottom-0 left-0 w-full">
      <canvas ref="mainImg" class="w-full opacity-75 h-[50vh]" />
      <div
        class="w-full h-full bottom-0 left-0 absolute bg-gradient-to-b from-white to-transparent z-10"
      ></div>
    </div>

    <div
      class="relative z-20 flex flex-col items-center justify-between h-[60%] pb-20"
    >
      <h1 class="text-sec-700 text-8xl philosopher-bold mb-20">ARCHPELAGUS</h1>

      <div
        class="flex gap-20 bg-white/30 py-5 px-10 rounded-lg items-center max-w-[900px]"
      >
        <input
          type="text"
          class="p-1 rounded-md text-black outline-none bg-white philosopher-regular-italic"
          v-model="userName"
          placeholder="Play as"
        />

        <Flags @selected="onSelected" class="flex-1" />

        <button
          class="bg-sec-700 text-white rounded-md philosopher-bold text-lg py-3 px-6"
          @click="onPlay"
        >
          Play
        </button>
      </div>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { getCurrentInstance, onMounted, ref } from "vue";
import archpelagus from "../assets/archpelagus.png";
import Flags from "../components/Flags.vue";
import { useRouter } from "vue-router";
import { ServerRequests } from "../requests/ServerRequests";

const router = useRouter();
const selectedFlag = ref<string | null>(null);
const userName = ref("");
const mainImg = ref();

ServerRequests.getServerList();

const instance = getCurrentInstance();
onMounted(() => {
  const canvas = mainImg.value;

  const img = new Image();
  img.src = archpelagus;

  const ctx = canvas.getContext("2d");
  canvas.width = window.innerWidth;
  let xTranslate = 0;
  const redrawLoop = () => {
    if (instance?.isUnmounted) {
      return;
    }

    canvas.height = img.height;
    const arrImgs = [img];
    arrImgs.forEach((img) => {
      const imgx = xTranslate;
      ctx.drawImage(
        img,
        0,
        0,
        img.width,
        img.height,
        imgx,
        0,
        img.width,
        img.height
      );
    });

    if (-xTranslate + canvas.width > img.width) {
      return;
    }
    requestAnimationFrame(redrawLoop);

    xTranslate -= 0.15;
  };

  img.onload = redrawLoop;
});

function onSelected(flag: string) {
  selectedFlag.value = flag;
}

function onPlay() {
  if (userName.value && selectedFlag.value) {
    router.push({
      path: "/game",
      query: {
        user: userName.value,
        flag: selectedFlag.value,
        online: "true",
      },
    });
  }
}
</script>

<style>
.philosopher-bold {
  font-family: "Philosopher", sans-serif;
  font-weight: 700;
  font-style: normal;
}

.philosopher-regular-italic {
  font-family: "Philosopher", sans-serif;
  font-weight: 400;
  font-style: italic;
}
</style>
