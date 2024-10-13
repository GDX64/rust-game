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
          ? 'opacity-100 scale-110 outline outline-2 outline-prime-200 -translate-y-1'
          : 'opacity-60',
      ]"
    />
  </div>
</template>

<script setup lang="ts">
import { ref } from "vue";
import { allCountries } from "../core/PlayerStuff";
import { useAsyncComputed } from "../utils/reactiveUtils";

const options = useAsyncComputed(async () => {
  const allOptions = Object.entries(allCountries);
  const options = [...Array(16)]
    .map(() => getRandom(allOptions))
    .filter((item) => item != null);

  const promises = options.map(async ([key, value]) => {
    const flagName = key.match(/\/(\w*)\.png/)?.[1];
    if (!flagName) {
      return null;
    }
    return {
      flagName,
      flag: await value(),
    };
  });
  const values = await Promise.all(promises);
  return values.filter((item) => item != null);
}, []);

const emit = defineEmits({
  selected: (flag: string) => flag,
});

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
