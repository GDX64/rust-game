<template>
  <div class="flex gap-5 w-full">
    <img
      :src="flag"
      v-for="(flag, index) in flags"
      @click="selected = index"
      :class="[
        'h-[35px] w-[50px] hover:opacity-100 ease-linear',
        selected === index ? 'opacity-100' : 'opacity-60',
      ]"
    />
  </div>
</template>

<script setup lang="ts">
import { ref } from "vue";
import ad from "../assets/flags/ad.png";
import ae from "../assets/flags/ae.png";
import af from "../assets/flags/af.png";
import ag from "../assets/flags/ag.png";
import ai from "../assets/flags/ai.png";
import al from "../assets/flags/al.png";
import am from "../assets/flags/am.png";
import ao from "../assets/flags/ao.png";
import aq from "../assets/flags/aq.png";
import ar from "../assets/flags/ar.png";

const flags = [ad, ae, af, ag, ai, al, am, ao, aq, ar];

const selected = ref(0);
</script>

<style scoped></style>
