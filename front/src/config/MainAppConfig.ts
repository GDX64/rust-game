declare const IS_PROD: boolean;
const isProd = IS_PROD;

export default {
  sockerUrl: isProd
    ? "wss://game.glmachado.com:5000/ws"
    : "ws://localhost:5000/ws",
};
