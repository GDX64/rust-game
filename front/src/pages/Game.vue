<template>
  <div class="" ref="container"></div>
</template>

<script lang="ts" setup>
import { onMounted, ref } from "vue";
import { ArchpelagusGame } from "../lib";

const container = ref<HTMLElement>();

onMounted(async () => {
  const game = await ArchpelagusGame.new(container.value!);
  console.log(game);
});
</script>
