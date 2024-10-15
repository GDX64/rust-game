<template>
  <div class="border border-sec-700 rounded-md p-2">
    <h3 class="font-bold">Ranking</h3>
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
          <td>{{ player.name }}</td>
          <td>{{ player.kills }}</td>
          <td>{{ player.deaths }}</td>
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
