import { createApp } from "vue";
import { createPinia } from "pinia";
import App from "./App.vue";
import { router } from "./router";
import { i18n } from "./i18n";
import "@fontsource/nunito/400.css";
import "@fontsource/nunito/600.css";
import "@fontsource/nunito/700.css";
import "@fontsource/nunito/800.css";
import "@fontsource/zen-maru-gothic/400.css";
import "@fontsource/zen-maru-gothic/500.css";
import "@fontsource/zen-maru-gothic/700.css";
import "./style.css";
import "./styles/markdown/index.css";
import "vue-sonner/style.css";

const app = createApp(App);
app.use(createPinia());
app.use(router);
app.use(i18n);
app.mount("#app");

// 仅在打包版本禁用 WebView 默认右键菜单;dev 保留以便右键检查元素调试。
// import.meta.env.DEV 由 Vite 静态替换,生产构建中此分支整体被消除,无运行时代价。
if (!import.meta.env.DEV) {
  window.addEventListener("contextmenu", (e) => e.preventDefault());
}
