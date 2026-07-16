import { createRouter, createWebHashHistory } from "vue-router";
import DetailPane from "@/components/layout/DetailPane.vue";

export const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    { path: "/", name: "home", component: DetailPane },
    { path: "/projects/:id", name: "project", component: DetailPane },
  ],
});

