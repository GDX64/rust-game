import { createApp } from "vue";
import { createWebHistory, createRouter, RouteRecordRaw } from "vue-router";
import App from "./App.vue";
import Home from "./pages/Home.vue";
import "./index.css";

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

start();

function start() {
  createApp(App).use(router).mount("#app");
}
