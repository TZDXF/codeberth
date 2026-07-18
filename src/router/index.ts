import { createRouter, createWebHashHistory } from "vue-router";
import ProjectsHome from "@/views/ProjectsHome.vue";
import ProjectDetail from "@/views/ProjectDetail.vue";

export const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    { path: "/", name: "home", component: ProjectsHome },
    { path: "/projects/:id", name: "project", component: ProjectDetail },
  ],
});
