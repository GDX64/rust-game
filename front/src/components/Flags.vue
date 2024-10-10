<template>
  <div
    class="flex gap-5 w-full overflow-x-auto overflow-y-hidden hide-scrollbar py-2"
  >
    <img
      :src="flag"
      v-for="(flag, index) in options"
      @click="selected = index"
      :class="[
        'max-h-[35px] hover:opacity-100 hover:scale-110 ease-linear transition-all rounded-md',
        selected === index
          ? 'opacity-100 scale-110 outline outline-white'
          : 'opacity-60',
      ]"
    />
  </div>
</template>

<script setup lang="ts">
import { ref } from "vue";
import { allCountries } from "../core/PlayerStuff";

const allArr = Object.values(allCountries);

const optionsPromises = [...Array(16)].map(() => getRandom(allArr));

const options = ref<string[]>([]);

optionsPromises.forEach(async (promise) => {
  options.value.push(await promise());
});

const selected = ref(0);

function getRandom<T>(arr: T[]) {
  return arr.splice(Math.floor(Math.random() * arr.length), 1)[0];
}
</script>

<style scoped></style>
