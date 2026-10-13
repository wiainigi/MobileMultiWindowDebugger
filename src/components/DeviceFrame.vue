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
  position: relative;
  display: flex;
  flex-direction: column;
  flex: none;
  border: 1px solid var(--border);
  border-radius: 16px;
  background: var(--bg-2);
  box-shadow: 0 14px 32px rgba(2, 5, 12, 0.55);
  overflow: hidden;
  transition: border-color 0.15s, box-shadow 0.15s;
}

.device-frame::after {
  content: "";
  position: absolute;
  inset: 0 0 auto;
  height: 2px;
  background: transparent;
  transition: background 0.15s;
}

.device-frame:hover {
  border-color: #33415f;
}

.device-frame.active {
  border-color: rgba(76, 141, 255, 0.7);
  box-shadow: 0 0 0 1px rgba(76, 141, 255, 0.3), 0 18px 42px rgba(2, 5, 12, 0.68);
}

.device-frame.active::after {
  background: linear-gradient(90deg, var(--accent), var(--accent-2));
}

.device-frame.incognito.active {
  border-color: rgba(124, 92, 255, 0.75);
  box-shadow: 0 0 0 1px rgba(124, 92, 255, 0.32), 0 18px 42px rgba(2, 5, 12, 0.68);
}

.device-frame.incognito.active::after {
  background: linear-gradient(90deg, var(--accent-2), #b06bff);
}

.frame-header {
  display: flex;
  align-items: center;
  gap: 8px;
  height: 36px;
  padding: 0 8px;
  background: var(--bg-3);
  border-bottom: 1px solid var(--border);
}

.incognito .frame-header {
  background: linear-gradient(180deg, rgba(124, 92, 255, 0.2), rgba(124, 92, 255, 0.05));
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
  background: #05070b;
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
  border-top: 1px solid var(--border);
  background: var(--bg-3);
  color: var(--fg-muted);
  font-size: 10.5px;
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
