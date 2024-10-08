import { createApp } from "vue";
import Game from "./pages/Game.vue";
import { createMemoryHistory, createRouter } from "vue-router";
import App from "./App.vue";
import Home from "./pages/Home.vue";
import "./index.css";

const routes = [
  { path: "/", component: Home },
  {
    path: "/game",
    component: Game,
  },
];

const router = createRouter({
  history: createMemoryHistory(),
  routes,
});

start();

function start() {
  createApp(App).use(router).mount("#app");
}

// import { Explosion } from "./Particles";
// Explosion.testRenderer();

// import { TestSphere } from "./TestSphere";
// TestSphere.testRenderer();
