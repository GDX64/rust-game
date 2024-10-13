<template>
  <div class="" ref="container"></div>
</template>

<script lang="ts" setup>
import { onMounted, onUnmounted, ref } from "vue";
import { ArchpelagusGame } from "../lib";
import { onBeforeRouteLeave, useRouter } from "vue-router";

const container = ref<HTMLElement>();
const router = useRouter();
onBeforeRouteLeave((guard) => {
  queueMicrotask(() => router.go(0));
});

let game: ArchpelagusGame | null = null;
onMounted(async () => {
  game = await ArchpelagusGame.new(container.value!);
});
onUnmounted(() => {
  game?.destroy();
});
</script>
