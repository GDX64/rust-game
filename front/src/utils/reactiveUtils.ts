import { shallowRef, watch } from "vue";

export function useAsyncComputed<T>(fn: () => Promise<T>, def: T) {
  const r = shallowRef(def);
  watch(
    fn,
    async (v) => {
      r.value = await v;
    },
    {
      immediate: true,
    }
  );
  return r;
}
