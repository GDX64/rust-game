<template>
  <div
    class="rounded-md border border-sec-700 w-64 bg-white/30 px-3 select-none"
  >
    <h2 class="font-bold py-1">Choose a Server:</h2>
    <div class="flex flex-col gap-1 pb-2">
      <div
        class="hover:bg-sec-200 rounded-md"
        v-for="server of servers"
        @click="emit('update:selected', server)"
      >
        <div
          class="grid grid-cols-[min-content_1fr_1fr_1fr] items-center gap-4"
        >
          <div
            class="w-4 h-4 rounded-full border-2 border-sec-600"
            :class="server.name === selected?.name ? 'bg-prime-200' : ''"
          ></div>
          <p>{{ server.name }}</p>
          <p class="self-end justify-self-end">{{ server.players }} players</p>
          <p class="self-end justify-self-end">seed {{ server.seed }}</p>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ServerList, ServerRequests } from "../requests/ServerRequests";
import { useAsyncComputed } from "../utils/reactiveUtils";

defineProps<{
  selected?: ServerList;
}>();

const emit = defineEmits({
  "update:selected": (selected: ServerList) => true,
});

const servers = useAsyncComputed(
  async () => ServerRequests.getServerList(),
  []
);
</script>
