declare const IS_PROD: boolean;

export const config = {
  preventDefaults: IS_PROD,
  isProd: IS_PROD,
  websocketURL: IS_PROD
    ? "https://archpelagus.glmachado.com/ws"
    : "ws://localhost:5000/ws",
  serverURL: IS_PROD
    ? "https://archpelagus.glmachado.com"
    : "http://localhost:5000",
};
