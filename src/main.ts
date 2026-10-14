import { createApp } from "vue";
import App from "./App.vue";
import "./styles/main.css";

// 屏蔽浏览器原生右键菜单（iframe 内的拦截由 Rust 侧初始化脚本完成）
window.addEventListener("contextmenu", (event) => event.preventDefault());

createApp(App).mount("#app");
