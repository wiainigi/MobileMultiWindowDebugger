<script setup lang="ts">
import { onMounted, ref } from "vue";

const appVersion = ref("-");
const appName = ref("MobileMultiWindowDebugger");

onMounted(async () => {
  try {
    const { getVersion, getName } = await import("@tauri-apps/api/app");
    appVersion.value = await getVersion();
    appName.value = await getName();
  } catch {
    appVersion.value = "dev";
  }
});
</script>

<template>
  <div class="mask" @click.self="$emit('close')">
    <div class="dialog">
      <div class="logo">◈</div>
      <h3>{{ appName }}</h3>
      <div class="version">v{{ appVersion }}</div>
      <ul class="tips">
        <li>无痕窗口使用独立沙箱源，Cookie 与本地存储不共享、关闭即销毁。</li>
        <li>受 <code>X-Frame-Options</code> / <code>CSP</code> 限制的站点无法在页内嵌入，可用「↗」按钮在默认浏览器中打开。</li>
        <li>快捷键：Ctrl+Shift+N 无痕窗口 · Ctrl+N 窗口 · Ctrl+W 关闭 · Ctrl+R 刷新。</li>
      </ul>
      <button class="close-btn" @click="$emit('close')">知道了</button>
    </div>
  </div>
</template>

<style scoped>
.mask {
  position: fixed;
  inset: 0;
  z-index: 80;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(5, 8, 14, 0.62);
  backdrop-filter: blur(3px);
}

.dialog {
  width: 400px;
  max-width: calc(100vw - 40px);
  padding: 26px 24px 20px;
  border: 1px solid var(--border);
  border-radius: 16px;
  background: var(--bg-2);
  box-shadow: 0 30px 70px rgba(0, 0, 0, 0.5);
  text-align: center;
}

.logo {
  font-size: 28px;
  color: var(--accent);
}

h3 {
  margin: 8px 0 2px;
  font-size: 16px;
}

.version {
  color: var(--fg-muted);
  font-size: 12px;
}

.tips {
  margin: 16px 0 18px;
  padding: 14px 16px;
  border: 1px solid var(--border);
  border-radius: 10px;
  background: var(--bg-0);
  text-align: left;
  list-style: none;
}

.tips li {
  position: relative;
  margin: 9px 0;
  padding-left: 16px;
  font-size: 12px;
  line-height: 1.75;
  color: var(--fg-dim);
}

.tips li::before {
  content: "";
  position: absolute;
  top: 8px;
  left: 2px;
  width: 5px;
  height: 5px;
  border-radius: 50%;
  background: var(--accent);
}

.tips li:first-child {
  margin-top: 0;
}

.tips li:last-child {
  margin-bottom: 0;
}

code {
  padding: 1px 4px;
  border-radius: 4px;
  background: var(--bg-3);
  color: #b9d2ff;
  font-size: 11px;
  white-space: nowrap;
}

.close-btn {
  width: 100%;
  height: 32px;
  border: none;
  border-radius: 9px;
  background: var(--accent);
  color: #fff;
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
}
</style>
