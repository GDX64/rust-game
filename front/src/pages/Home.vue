<template>
  <div
    class="h-full w-full flex flex-col philosopher overflow-x- px-4 overflow-y-auto pt-16 pb-20"
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
      class="z-10 flex flex-col items-center justify-between grow gap-8 min-h-fit"
    >
      <h1 class="text-sec-700 text-4xl sm:text-7xl font-bold">ARCHPELAGUS</h1>

      <div class="flex gap-8 flex-wrap justify-center items-center w-full">
        <Ranking class="w-full sm:w-auto"></Ranking>
        <div class="flex flex-col gap-2 w-full sm:w-auto">
          <ServerSelector
            class="w-full"
            v-model:selected="serverSelected"
            :class="online ? '' : 'opacity-30 pointer-events-none'"
          ></ServerSelector>
          <label class="px-1 flex gap-2 select-none">
            <input type="checkbox" v-model="online" />
            <div class="">Online</div>
          </label>
        </div>
      </div>

      <div
        class="flex gap-4 flex-col min-h-fit sm:flex-row items-start sm:items-center bg-white/30 p-3 rounded-md max-w-[900px] border border-sec-700 z-10 overflow-hidden w-full"
      >
        <input
          type="text"
          class="block p-1 rounded-md text-black outline-none bg-sec-100 philosopher-regular-italic focus:outline-prime-200 outline-offset-0 w-full sm:w-auto"
          v-model="userName"
          placeholder="Play as"
        />

        <Flags @selected="onSelected" class="flex-1" />

        <button
          class="bg-sec-700 text-white rounded-md text-lg py-3 px-6 disabled:opacity-30 w-full sm:w-auto"
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
import Ranking from "./Ranking.vue";

const router = useRouter();
const selectedFlag = ref<string | null>(null);
const userName = ref("");
const online = ref(true);

const serverSelected = ref<string>();

function canPlay() {
  if (
    userName.value &&
    selectedFlag.value &&
    (serverSelected.value || !online.value)
  ) {
    return {
      user: userName.value,
      flag: selectedFlag.value,
      server_id: serverSelected.value,
      online: online.value,
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
  let id;
  if (data.online) {
    if (data.server_id) {
      id = await ServerRequests.getPlayerID(data.server_id);
    } else {
      throw new Error("Server ID not found");
    }
  } else {
    id = 0;
  }
  if (canPlay()) {
    router.push({
      path: "/game",
      query: {
        player_name: userName.value,
        flag: selectedFlag.value,
        server_id: serverSelected.value,
        player_id: id,
        online: data.online ? "true" : "false",
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
