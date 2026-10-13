<script setup lang="ts">
import { computed } from "vue";
import type { DeviceView } from "../types";
import { DEVICE_GROUPS, findDevice } from "../constants/devices";

const props = defineProps<{
  view: DeviceView;
  index: number;
  scale: number;
  active: boolean;
}>();

const emit = defineEmits<{
  (e: "activate"): void;
  (e: "update", patch: Partial<DeviceView>): void;
  (e: "close"): void;
}>();

const device = computed(() => findDevice(props.view.deviceId));

const logicalSize = computed(() =>
  props.view.orientation === "landscape"
    ? { width: device.value.height, height: device.value.width }
    : { width: device.value.width, height: device.value.height },
);

const viewportStyle = computed(() => ({
  width: `${logicalSize.value.width * props.scale}px`,
  height: `${logicalSize.value.height * props.scale}px`,
}));

const screenStyle = computed(() => ({
  width: `${logicalSize.value.width}px`,
  height: `${logicalSize.value.height}px`,
  transform: `scale(${props.scale})`,
}));

/**
 * 无痕窗口不授予 allow-same-origin，iframe 将获得独立的不透明源：
 * Cookie / localStorage / IndexedDB 与常规窗口完全隔离，关闭即销毁。
 */
const sandboxValue = computed(() =>
  props.view.incognito
    ? "allow-scripts allow-forms allow-popups allow-popups-to-escape-sandbox allow-modals allow-downloads"
    : "allow-scripts allow-same-origin allow-forms allow-popups allow-popups-to-escape-sandbox allow-modals allow-downloads",
);

function changeDevice(event: Event) {
  emit("update", { deviceId: (event.target as HTMLSelectElement).value });
}

function toggleOrientation() {
  emit("update", { orientation: props.view.orientation === "portrait" ? "landscape" : "portrait" });
}

function reload() {
  emit("update", { reloadToken: props.view.reloadToken + 1 });
}

async function openExternal() {
  try {
    const { openUrl } = await import("@tauri-apps/plugin-opener");
    await openUrl(props.view.url);
  } catch {
    window.open(props.view.url, "_blank");
  }
}
</script>

<template>
  <article class="device-frame" :class="{ active, incognito: view.incognito }" @mousedown="emit('activate')">
    <header class="frame-header">
      <span class="index-badge" :title="view.label">{{ index }}</span>

      <div class="frame-meta">
        <span class="frame-title">{{ view.label }}</span>
        <span v-if="view.incognito" class="badge">无痕</span>
      </div>

      <select class="frame-select" :value="view.deviceId" title="切换机型分辨率" @change="changeDevice" @mousedown.stop="emit('activate')">
        <optgroup v-for="group in DEVICE_GROUPS" :key="group.name" :label="group.name">
          <option v-for="preset in group.devices" :key="preset.id" :value="preset.id">
            {{ preset.name }} · {{ preset.width }}×{{ preset.height }}
          </option>
        </optgroup>
      </select>

      <span class="size-tag">{{ logicalSize.width }}×{{ logicalSize.height }}</span>

      <div class="frame-actions">
        <button class="icon-btn" title="旋转屏幕" @mousedown.stop="emit('activate')" @click="toggleOrientation">⟲</button>
        <button class="icon-btn" title="重新加载" @mousedown.stop="emit('activate')" @click="reload">⟳</button>
        <button class="icon-btn" title="在默认浏览器中打开" @mousedown.stop="emit('activate')" @click="openExternal">↗</button>
        <button class="icon-btn danger" title="关闭窗口" @mousedown.stop="emit('activate')" @click="emit('close')">✕</button>
      </div>
    </header>

    <div class="frame-viewport" :style="viewportStyle">
      <div class="frame-screen" :style="screenStyle">
        <iframe
          :key="`${view.id}-${view.reloadToken}`"
          class="frame-iframe"
          :data-view-id="view.id"
          :src="view.url"
          :sandbox="sandboxValue"
          referrerpolicy="no-referrer-when-downgrade"
          allow="geolocation; camera; microphone; fullscreen; accelerometer; gyroscope; clipboard-write"
        />
      </div>
    </div>

    <footer class="frame-footer">
      <span class="footer-dot" />
      <span class="footer-url" :title="view.url">{{ view.url }}</span>
      <span class="footer-scale">{{ Math.round(scale * 100) }}%</span>
    </footer>
  </article>
</template>

<style scoped>
.device-frame {
  /* 普通窗口：黑蓝配色 */
  --frame-bg: #0c1422;
  --frame-bg-active: #16243c;
  --frame-screen: #04070d;
  --frame-border: #223049;
  --frame-border-hover: #3b4a6d;
  --frame-glow: rgba(76, 141, 255, 0.55);
  --frame-chrome: rgba(255, 255, 255, 0.04);
  --frame-chrome-active: rgba(255, 255, 255, 0.075);

  position: relative;
  display: flex;
  flex-direction: column;
  flex: none;
  border: 1px solid var(--frame-border);
  border-radius: 16px;
  background: var(--frame-bg);
  box-shadow: 0 14px 32px rgba(2, 5, 12, 0.55);
  overflow: hidden;
  transition: background 0.16s, border-color 0.16s, box-shadow 0.16s;
}

/* 无痕窗口：黑紫配色 */
.device-frame.incognito {
  --frame-bg: #130e26;
  --frame-bg-active: #241a46;
  --frame-screen: #08050f;
  --frame-border: #2b2148;
  --frame-border-hover: #453571;
  --frame-glow: rgba(124, 92, 255, 0.6);
}

.device-frame:hover:not(.active) {
  border-color: var(--frame-border-hover);
}

/* 活动窗口：同色系中更亮一档的底色 */
.device-frame.active {
  background: var(--frame-bg-active);
  border-color: var(--frame-glow);
  box-shadow: 0 0 0 1px var(--frame-glow), 0 18px 42px rgba(2, 5, 12, 0.68);
}

.frame-header {
  display: flex;
  align-items: center;
  gap: 8px;
  height: 36px;
  padding: 0 8px;
  background: var(--frame-chrome);
  border-bottom: 1px solid var(--frame-border);
}

.active .frame-header {
  background: var(--frame-chrome-active);
}

.index-badge {
  display: grid;
  place-items: center;
  flex: none;
  width: 18px;
  height: 18px;
  border-radius: 6px;
  background: var(--bg-0);
  border: 1px solid var(--border);
  color: var(--fg-dim);
  font-size: 10px;
  font-weight: 700;
}

.active .index-badge {
  border-color: rgba(76, 141, 255, 0.6);
  color: #b9d2ff;
}

.incognito.active .index-badge {
  border-color: rgba(124, 92, 255, 0.6);
  color: #cbbcff;
}

.frame-meta {
  display: flex;
  align-items: center;
  gap: 6px;
  min-width: 0;
}

.frame-title {
  overflow: hidden;
  font-size: 12px;
  font-weight: 600;
  white-space: nowrap;
  text-overflow: ellipsis;
}

.badge {
  flex: none;
  padding: 1px 6px;
  border-radius: 999px;
  background: rgba(124, 92, 255, 0.24);
  color: #cbbcff;
  font-size: 10px;
  font-weight: 600;
}

.frame-select {
  max-width: 120px;
  height: 22px;
  margin-left: auto;
  padding: 0 4px;
  border: 1px solid var(--border);
  border-radius: 6px;
  background: var(--bg-0);
  color: var(--fg-dim);
  font-size: 11px;
  cursor: pointer;
}

.size-tag {
  flex: none;
  color: var(--fg-muted);
  font-size: 11px;
  font-variant-numeric: tabular-nums;
  white-space: nowrap;
}

.frame-actions {
  display: flex;
  gap: 2px;
}

.icon-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 22px;
  height: 22px;
  border: none;
  border-radius: 6px;
  background: transparent;
  color: var(--fg-dim);
  font-size: 12px;
  cursor: pointer;
}

.icon-btn:hover {
  background: var(--bg-hover);
  color: var(--fg);
}

.icon-btn.danger:hover {
  background: rgba(255, 92, 108, 0.16);
  color: #ff8b90;
}

.frame-viewport {
  position: relative;
  flex: none;
  background: var(--frame-screen);
  overflow: hidden;
}

.frame-screen {
  position: absolute;
  top: 0;
  left: 0;
  transform-origin: top left;
  border-radius: 4px;
  overflow: hidden;
  box-shadow: inset 0 0 0 1px rgba(255, 255, 255, 0.05);
}

.frame-iframe {
  display: block;
  width: 100%;
  height: 100%;
  border: none;
  background: #fff;
}

.frame-footer {
  display: flex;
  align-items: center;
  gap: 6px;
  height: 26px;
  padding: 0 9px;
  border-top: 1px solid var(--frame-border);
  background: var(--frame-chrome);
  color: var(--fg-muted);
  font-size: 10.5px;
}

.active .frame-footer {
  background: var(--frame-chrome-active);
}

.footer-dot {
  flex: none;
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: #3ddc97;
}

.incognito .footer-dot {
  background: var(--accent-2);
}

.footer-url {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  font-family: "Cascadia Mono", "Consolas", monospace;
  white-space: nowrap;
  text-overflow: ellipsis;
}

.footer-scale {
  flex: none;
  padding: 1px 6px;
  border: 1px solid var(--border);
  border-radius: 999px;
  color: var(--fg-dim);
  font-variant-numeric: tabular-nums;
}
</style>
