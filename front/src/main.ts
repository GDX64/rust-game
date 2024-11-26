import { createApp } from "vue";
import { createWebHistory, createRouter, RouteRecordRaw } from "vue-router";
import App from "./App.vue";
import Home from "./pages/Home.vue";
import "./index.css";
import { ServerRequests } from "./requests/ServerRequests";

const GAME_ROUTE = "/game";
const routes: RouteRecordRaw[] = [
  { path: "/", component: Home },
  {
    path: GAME_ROUTE,
    component: () => import("./pages/Game.vue"),
  },
];

const router = createRouter({
  history: createWebHistory(),
  routes,
});

router.afterEach((to, from) => {
  if (from.path === GAME_ROUTE && to.path !== GAME_ROUTE) {
    router.go(0);
  }
});

window.addEventListener("error", (event) => {
  ServerRequests.sendError(event.error);
});

window.addEventListener("unhandledrejection", (event) => {
  ServerRequests.sendError(event.reason);
});

start();

function start() {
  createApp(App).use(router).mount("#app");
}
