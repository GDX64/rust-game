<template>
  <input
    type="text"
    @input="emit('update:value', ($event.target as any).value)"
    :value="value"
    class="block disabled:opacity-30 border border-sec-500 p-1 rounded-md text-black outline-none bg-sec-100 philosopher-regular-italic focus:outline-prime-200 outline-offset-0 w-full sm:w-auto"
  />
</template>

<script setup lang="ts">
defineProps<{
  value: string;
}>();

const emit = defineEmits({
  "update:value": (value: string) => true,
});
</script>
