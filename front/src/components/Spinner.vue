<template>
  <div class="h-full w-full flex items-center justify-center flex-col">
    <div
      v-html="Boat"
      class="w-12 h-12 [&_svg]:w-full [&_svg]:h-full [&_path]:fill-black wave-animation"
    ></div>
    <div
      class="w-8 h-8 text-black [&_svg]:w-full [&_svg]:h-full"
      v-html="spinner"
    ></div>
  </div>
</template>

<script setup lan="ts">
import Boat from "../assets/icon.svg?raw";
import spinner from "../assets/spinner.svg?raw";
</script>

<style scoped>
@keyframes bounce {
  0% {
    transform: translateY(0);
  }
  100% {
    transform: translateY(-25%);
  }
}
.wave-animation {
  animation: bounce 0.5s infinite alternate ease-in-out;
  animation-delay: 0.25s;
}
</style>
