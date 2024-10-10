import { createApp } from "vue";
import { createWebHistory, createRouter } from "vue-router";
import App from "./App.vue";
import Home from "./pages/Home.vue";
import "./index.css";

const routes = [
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

// router.afterEach((to, from, next) => {
//   if (to.path != from.path) {
//     window.location.reload();
//   } else {
//     // next();
//   }
// });

start();

function start() {
  createApp(App).use(router).mount("#app");
}
