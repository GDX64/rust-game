import { createApp } from "vue";
import { createWebHistory, createRouter, RouteRecordRaw } from "vue-router";
import App from "./App.vue";
import Home from "./pages/Home.vue";
import "./index.css";
import { ServerRequests } from "./requests/ServerRequests";

const routes: RouteRecordRaw[] = [
  { path: "/", component: Home },
  {
    path: "/game",
    component: () => import("./pages/Game.vue"),
  },
];

const router = createRouter({
  history: createWebHistory(),
  routes,
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
