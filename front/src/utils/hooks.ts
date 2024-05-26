import {
  computed,
  onUnmounted,
  reactive,
  ref,
  watchEffect,
  watchSyncEffect,
} from "vue";

export function useCanvasDPI() {
  const canvas = ref<HTMLCanvasElement>();
  const { size } = useSize(canvas);
  watchSyncEffect(() => {
    if (canvas.value) {
      const ctx = canvas.value.getContext("2d");
      if (ctx) {
        const dpr = self.devicePixelRatio || 1;
        const { width, height } = size;
        canvas.value.width = Math.floor(width * dpr);
        canvas.value.height = Math.floor(height * dpr);
      }
    }
  });
  return {
    canvas,
    size,
    pixelSize: computed(() => {
      return {
        width: Math.floor(size.width * (self.devicePixelRatio || 1)),
        height: Math.floor(size.height * (self.devicePixelRatio || 1)),
      };
    }),
  };
}

export function useSize(container = ref<HTMLElement | null>()) {
  const size = reactive({ width: 0, height: 0 });
  const obs = new ResizeObserver((_entries) => {
    const el = container.value;
    size.width = el?.clientWidth ?? 0;
    size.height = el?.clientHeight ?? 0;
  });
  watchEffect((clear) => {
    const el = container.value;
    if (el) {
      obs.observe(el);
      clear(() => obs.unobserve(el));
    }
  });
  onUnmounted(() => obs.disconnect());
  return { size, container };
}

export function useAnimationFrames(
  fn: (args: { elapsed: number; delta: number; count: number }) => void
) {
  let last = performance.now();
  let count = 0;
  let reqID = -1;

  function onFrame() {
    const elapsed = performance.now();
    const delta = elapsed - last;
    last = elapsed;
    count++;
    fn({ elapsed, delta, count });
    reqID = requestAnimationFrame(onFrame);
  }

  reqID = requestAnimationFrame(onFrame);
  onUnmounted(() => {
    cancelAnimationFrame(reqID);
  });
}
