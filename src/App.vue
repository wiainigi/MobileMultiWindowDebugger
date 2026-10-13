<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from "vue";
import MenuBar from "./components/MenuBar.vue";
import ToolBar from "./components/ToolBar.vue";
import DeviceStage from "./components/DeviceStage.vue";
import AboutDialog from "./components/AboutDialog.vue";
import { deviceManager } from "./composables/useDeviceManager";
import { findDevice } from "./constants/devices";

const showAbout = ref(false);

onMounted(() => {
  if (deviceManager.views.value.length === 0) {
    deviceManager.createView({ incognito: false, label: "初始窗口" });
  }
  window.addEventListener("keydown", onKeydown);
});

onBeforeUnmount(() => window.removeEventListener("keydown", onKeydown));

function onKeydown(event: KeyboardEvent) {
  if (!(event.ctrlKey || event.metaKey)) return;
  const key = event.key.toLowerCase();
  if (key === "n" && event.shiftKey) {
    event.preventDefault();
    deviceManager.createIncognito();
  } else if (key === "n") {
    event.preventDefault();
    deviceManager.createWindow();
  } else if (key === "w") {
    event.preventDefault();
    deviceManager.closeActive();
  } else if (key === "r") {
    event.preventDefault();
    deviceManager.reload();
  }
}

const stageScale = computed(() => Math.round(deviceManager.stageScale.value * 100));

const statusText = computed(() => {
  const view = deviceManager.activeView.value;
  if (!view) return "无活动窗口";
  const device = findDevice(view.deviceId);
  const size = view.orientation === "landscape" ? `${device.height}×${device.width}` : `${device.width}×${device.height}`;
  return `${view.label} · ${device.name} · ${size} · ${view.orientation === "portrait" ? "竖屏" : "横屏"}`;
});
</script>

<template>
  <div class="app-shell">
    <MenuBar @open-about="showAbout = true" />
    <ToolBar />
    <DeviceStage />

    <footer class="status-bar">
      <span class="status-item active-dot" />
      <span class="status-item">{{ statusText }}</span>
      <span class="status-item grow">提示：点击任意窗口可将其设为活动窗口，地址栏操作只作用于活动窗口。</span>
      <span class="status-item">缩放 {{ stageScale }}%</span>
      <span class="status-item">窗口 {{ deviceManager.views.value.length }} · 无痕 {{ deviceManager.incognitoCount.value }}</span>
    </footer>

    <AboutDialog v-if="showAbout" @close="showAbout = false" />
  </div>
</template>

<style scoped>
.app-shell {
  display: flex;
  flex-direction: column;
  height: 100vh;
  overflow: hidden;
}

.status-bar {
  display: flex;
  align-items: center;
  gap: 12px;
  height: 26px;
  padding: 0 12px;
  background: var(--bg-1);
  border-top: 1px solid var(--border);
  color: var(--fg-muted);
  font-size: 11px;
  white-space: nowrap;
}

.status-item {
  overflow: hidden;
  text-overflow: ellipsis;
}

.grow {
  flex: 1;
  text-align: right;
}

.active-dot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  background: #3ddc97;
  flex: none;
}
</style>
