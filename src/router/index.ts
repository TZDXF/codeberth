import { createRouter, createWebHashHistory } from "vue-router";
import ProjectsHome from "@/views/ProjectsHome.vue";
import ProjectDetail from "@/views/ProjectDetail.vue";
import Settings from "@/views/Settings.vue";
import ReportHistory from "@/views/ReportHistory.vue";

export const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    { path: "/", name: "home", component: ProjectsHome },
    { path: "/projects/:id", name: "project", component: ProjectDetail },
    { path: "/settings", name: "settings", component: Settings },
    { path: "/report-history", name: "history", component: ReportHistory },
  ],
});
