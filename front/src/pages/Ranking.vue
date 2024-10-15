<template>
  <div
    class="border border-sec-700 rounded-md p-2 max-h-[300px] overflow-y-auto overflow-x-hidden"
  >
    <h3 class="font-bold pb-1">Week Ranking</h3>
    <table class="">
      <thead>
        <tr class="bg-sec-300">
          <th class="min-w-16 text-start">Player</th>
          <th class="min-w-16 text-start">Kills</th>
          <th class="min-w-16 text-start">Deaths</th>
        </tr>
      </thead>
      <tbody>
        <tr
          v-for="player in ranking"
          :key="player.name"
          class="even:bg-sec-200"
        >
          <td class="px-2">{{ player.name }}</td>
          <td class="px-2">{{ player.kills }}</td>
          <td class="px-2">{{ player.deaths }}</td>
        </tr>
      </tbody>
    </table>
  </div>
</template>
<script lang="ts" setup>
import { ServerRequests } from "../requests/ServerRequests";
import { useAsyncComputed } from "../utils/reactiveUtils";

const ranking = useAsyncComputed(async () => ServerRequests.getRanking(), []);
</script>
