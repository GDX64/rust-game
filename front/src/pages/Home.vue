<template>
  <div
    class="h-full w-full flex items-center justify-center flex-col philosopher"
  >
    <div
      class="absolute -z-1 w-full h-[30%] top-0 left-0 bg-gradient-to-b from-prime-100 to-white opacity-80"
    ></div>
    <div class="absolute -z-1 bottom-0 left-0 w-full">
      <img :src="archpelagus" class="w-full opacity-75 h-[50vh] object-cover" />
      <div
        class="w-full h-full bottom-0 left-0 absolute bg-gradient-to-b from-white to-transparent z-10"
      ></div>
    </div>

    <div
      class="relative z-10 flex flex-col items-center justify-between h-full pt-16 pb-20"
    >
      <h1 class="text-sec-700 text-8xl font-bold">ARCHPELAGUS</h1>

      <ServerSelector v-model:selected="serverSelected"></ServerSelector>

      <div
        class="flex gap-20 bg-white/30 py-5 px-10 rounded-md items-center max-w-[900px] border border-sec-700 z-10"
      >
        <input
          type="text"
          class="p-1 rounded-md text-black outline-none bg-white philosopher-regular-italic focus:outline-prime-200 outline-offset-0"
          v-model="userName"
          placeholder="Play as"
        />

        <Flags @selected="onSelected" class="flex-1" />

        <button
          class="bg-sec-700 text-white rounded-md text-lg py-3 px-6 disabled:opacity-30"
          :disabled="!canPlay()"
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
import ServerSelector from "./ServerSelector.vue";
import { ServerRequests } from "../requests/ServerRequests";

const router = useRouter();
const selectedFlag = ref<string | null>(null);
const userName = ref("");

const serverSelected = ref<string>();

function canPlay() {
  if (userName.value && selectedFlag.value && serverSelected.value) {
    return {
      user: userName.value,
      flag: selectedFlag.value,
      server_id: serverSelected.value,
    };
  }
  return null;
}

function onSelected(flag: string) {
  selectedFlag.value = flag;
}

async function onPlay() {
  const data = canPlay();
  if (!data) {
    return;
  }
  const id = await ServerRequests.getPlayerID(data.server_id);
  if (canPlay()) {
    router.push({
      path: "/game",
      query: {
        user: userName.value,
        flag: selectedFlag.value,
        server_id: serverSelected.value,
        player_id: id,
        online: "true",
      },
    });
  }
}
</script>

<style>
.philosopher {
  font-family: "Philosopher", sans-serif;
  font-weight: 400;
  font-style: normal;
}

.philosopher-regular-italic {
  font-family: "Philosopher", sans-serif;
  font-weight: 400;
  font-style: italic;
}
</style>
