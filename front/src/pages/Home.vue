<template>
  <div class="h-full w-full flex items-end justify-center">
    <div
      class="absolute -z-1 w-full h-[30%] top-0 left-0 bg-gradient-to-b from-yellow-100 to-white opacity-80"
    ></div>
    <div class="absolute -z-1 bottom-0 left-0 w-full">
      <img :src="archpelagus" class="w-full opacity-75" />
      <div
        class="w-full h-full bottom-0 left-0 absolute bg-gradient-to-b from-white to-transparent z-10"
      ></div>
    </div>

    <div
      class="relative z-20 flex flex-col items-center justify-between h-[60%] pb-20"
    >
      <h1 class="text-gray-700 text-8xl philosopher-bold mb-20">ARCHPELAGUS</h1>

      <div class="flex gap-20 bg-white/30 py-5 px-10 rounded-lg items-center">
        <input
          type="text"
          class="p-1 rounded-md text-black outline-none bg-white philosopher-regular-italic"
          v-model="userName"
          placeholder="Play as"
        />

        <Flags @selected="onSelected" class="flex-1" />

        <button
          class="bg-warmGray-600 text-white rounded-md philosopher-bold text-lg py-3 px-6"
          @click="onPlay"
        >
          Play
        </button>
      </div>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { ref } from "vue";
import archpelagus from "../assets/archpelagus.png";
import Flags from "../components/Flags.vue";
import { useRouter } from "vue-router";

const router = useRouter();
const selectedFlag = ref<string | null>(null);
const userName = ref("");

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
