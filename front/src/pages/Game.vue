<template>
  <div class="w-full relative h-screen">
    <Transition>
      <Spinner v-if="!game" class="bg-sec-100 z-10 absolute"></Spinner>
    </Transition>
    <div v-show="game" ref="container"></div>
    <!-- <button
      class="absolute z-10 bottom-0 left-0 rounded-full bg-prime-200 text-black"
    >
      Audio
    </button> -->
  </div>
</template>

<script lang="ts" setup>
import { onMounted, onUnmounted, ref, shallowRef } from "vue";
import { ArchpelagusGame } from "../lib";
import { onBeforeRouteLeave } from "vue-router";
import Spinner from "../components/Spinner.vue";
import { awaitTime } from "../utils/promiseUtils";

const container = ref<HTMLElement>();

const game = shallowRef<ArchpelagusGame>();

onMounted(async () => {
  if (container.value) {
    const time = awaitTime(1_000);
    const [gameObj, _] = await Promise.all([
      ArchpelagusGame.new(container.value),
      time,
    ]);
    game.value = gameObj;
  }
});

onUnmounted(() => {
  game.value?.destroy();
});
</script>

<style scoped>
.v-enter-active,
.v-leave-active {
  transition: opacity 1s ease-out;
}

.v-enter-from,
.v-leave-to {
  opacity: 0;
}
</style>
