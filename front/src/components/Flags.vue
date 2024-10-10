<template>
  <div
    class="flex gap-5 w-full overflow-x-auto overflow-y-hidden hide-scrollbar py-2 px-2"
  >
    <img
      :src="flag"
      v-for="flag in options"
      @click="onSelect(flag)"
      :class="[
        'max-h-[35px] hover:opacity-100 hover:scale-110 ease-linear transition-all rounded-md',
        selected === flag
          ? 'opacity-100 scale-110 outline outline-high-500'
          : 'opacity-60',
      ]"
    />
  </div>
</template>

<script setup lang="ts">
import { ref } from "vue";
import { allCountries } from "../core/PlayerStuff";

const allArr = Object.values(allCountries);

const emit = defineEmits({
  selected: (flag: string) => flag,
});

const optionsPromises = [...Array(16)].map(() => getRandom(allArr));

const options = ref<string[]>([]);

optionsPromises.forEach(async (promise) => {
  options.value.push(await promise());
});

const selected = ref<null | string>(null);

function onSelect(flag: string) {
  selected.value = flag;
  const flagName = flag.match(/\/(\w*)\.png/)?.[1];
  if (flagName) {
    emit("selected", flagName);
  }
}

function getRandom<T>(arr: T[]) {
  return arr.splice(Math.floor(Math.random() * arr.length), 1)[0];
}
</script>

<style scoped></style>
