<template>
  <div
    class="flex gap-5 w-full overflow-x-auto overflow-y-hidden hide-scrollbar py-2 px-2"
  >
    <img
      :src="flag"
      v-for="{ flag, flagName } in options"
      @click="onSelect(flagName)"
      :class="[
        'max-h-[35px] hover:opacity-100 hover:scale-110 ease-linear transition-all rounded-md',
        selected === flagName
          ? 'opacity-100 scale-110 outline outline-high-500'
          : 'opacity-60',
      ]"
    />
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from "vue";
import { allCountries } from "../core/PlayerStuff";
import { useAsyncComputed } from "../utils/reactiveUtils";

const allArr = useAsyncComputed(() => {
  const promises = Object.entries(allCountries).map(async ([key, value]) => {
    const flagName = key.match(/\/(\w*)\.png/)?.[1];
    if (!flagName) {
      return null;
    }
    return {
      flagName,
      flag: await value(),
    };
  });
  return Promise.all(promises);
}, []);

const emit = defineEmits({
  selected: (flag: string) => flag,
});

const options = computed(() =>
  [...Array(16)]
    .map(() => getRandom(allArr.value))
    .filter((item) => item != null)
);

const selected = ref<null | string>(null);

function onSelect(flag: string) {
  selected.value = flag;
  emit("selected", flag);
}

function getRandom<T>(arr: T[]) {
  return arr.splice(Math.floor(Math.random() * arr.length), 1)[0];
}
</script>

<style scoped></style>
