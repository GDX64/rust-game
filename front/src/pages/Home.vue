<template>
  <div
    class="h-full w-full flex flex-col philosopher overflow-x- px-4 overflow-y-auto pt-16 pb-20 min-h-screen"
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
          <div class="px-1 flex gap-2 select-none items-center w-max">
            <label class="flex items-center w-max gap-2">
              <input type="checkbox" v-model="online" />
              <div class="">Online</div>
            </label>
            <TextInput
              v-model:value="localSeed"
              placeholder="Seed, ex: 123"
              :disabled="online"
            ></TextInput>
          </div>
        </div>
      </div>

      <div
        class="flex gap-4 flex-col min-h-fit sm:flex-row items-start sm:items-center bg-white/30 p-3 rounded-md max-w-[900px] border border-sec-700 z-10 overflow-hidden w-full"
      >
        <TextInput
          v-model:value="userName"
          placeholder="Play as..."
        ></TextInput>

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
import { ServerList, ServerRequests } from "../requests/ServerRequests";
import Ranking from "./Ranking.vue";
import TextInput from "./TextInput.vue";

const router = useRouter();
const selectedFlag = ref<string | null>(null);
const userName = ref("");
const online = ref(true);
const localSeed = ref("");

const serverSelected = ref<ServerList>();

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
      id = await ServerRequests.getPlayerID(data.server_id.name);
    } else {
      throw new Error("Server ID not found");
    }
  } else {
    id = 0;
  }
  if (canPlay()) {
    const seed = data.online
      ? serverSelected.value?.seed
      : Number(localSeed.value) || 0;

    if (seed == undefined) {
      throw new Error("Seed is undefined");
    }

    router.push({
      path: "/game",
      query: {
        player_name: userName.value,
        flag: selectedFlag.value,
        server_id: serverSelected.value?.name,
        seed,
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
</style>
