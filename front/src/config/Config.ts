declare const IS_PROD: boolean;

export const config = {
  preventDefaults: IS_PROD,
  serverURL: IS_PROD
    ? "https://archpelagus.glmachado.com/ws"
    : "ws://localhost:5000/ws",
};
