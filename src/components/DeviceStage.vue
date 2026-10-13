<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, reactive, ref, watch } from "vue";
import { deviceManager } from "../composables/useDeviceManager";
import { findDevice } from "../constants/devices";
import type { DeviceView } from "../types";
import DeviceFrame from "./DeviceFrame.vue";

/** 舞台内边距 */
const PADDING = 24;
/** 窗口之间的间距 */
const GAP = 18;
/** 单个窗口除屏幕外的固定高度：标题栏 + 页脚 + 边框 */
const FRAME_CHROME_HEIGHT = 68;
/** 窗口卡片的最小可读宽度，缩放到此为止 */
const MIN_FRAME_WIDTH = 224;

const stageRef = ref<HTMLElement | null>(null);
const stageSize = reactive({ width: 0, height: 0 });
let observer: ResizeObserver | null = null;

/**
 * 点击 iframe 内容区域时，鼠标事件不会冒泡到宿主页面，
 * 因此在捕获阶段监听 focus —— 跨源 iframe 被点击时
 * 其 <iframe> 元素自身会在宿主文档中触发 focus 事件。
 */
function handleFocusIn(event: Event) {
  const target = event.target;
  if (!(target instanceof HTMLIFrameElement)) return;
  const viewId = target.dataset["viewId"];
  if (viewId && views.value.some((view) => view.id === viewId)) {
    deviceManager.setActive(viewId);
  }
}

onMounted(() => {
  if (!stageRef.value) return;
  const rect = stageRef.value.getBoundingClientRect();
  stageSize.width = rect.width;
  stageSize.height = rect.height;

  observer = new ResizeObserver((entries) => {
    const content = entries[0].contentRect;
    stageSize.width = content.width;
    stageSize.height = content.height;
  });
  observer.observe(stageRef.value);

  window.addEventListener("focus", handleFocusIn, true);
  document.addEventListener("focusin", handleFocusIn);
});

onBeforeUnmount(() => {
  observer?.disconnect();
  window.removeEventListener("focus", handleFocusIn, true);
  document.removeEventListener("focusin", handleFocusIn);
});

const views = computed<DeviceView[]>(() => deviceManager.views.value);
const layout = computed(() => deviceManager.layout.value);
const activeId = computed(() => deviceManager.activeId.value);

function logicalSize(view: DeviceView) {
  const device = findDevice(view.deviceId);
  return view.orientation === "landscape"
    ? { width: device.height, height: device.width }
    : { width: device.width, height: device.height };
}

function onUpdate(id: string, patch: Partial<DeviceView>) {
  deviceManager.patch(id, patch);
}

/**
 * 新窗口加入后自动压缩每个窗口的显示比例：
 * 同时受舞台宽度与高度约束，保证窗口完整可见；
 * 缩放下限用于保证窗口卡片仍然可读、可操作，超出时舞台横向滚动。
 */
const scale = computed(() => {
  const list = views.value;
  if (list.length === 0 || !deviceManager.fitToStage.value) return 1;

  const availWidth = Math.max(0, stageSize.width - PADDING * 2);
  const availHeight = Math.max(0, stageSize.height - PADDING * 2);
  if (availWidth === 0 || availHeight === 0) return 1;

  const sizes = list.map(logicalSize);
  const maxWidth = Math.max(...sizes.map((size) => size.width));
  const maxHeight = Math.max(...sizes.map((size) => size.height));
  const minScale = Math.min(1, MIN_FRAME_WIDTH / maxWidth);

  const totalWidth =
    sizes.reduce((sum, size) => sum + size.width, 0) +
    (layout.value === "row" ? (list.length - 1) * GAP : 0);
  const ratio = Math.min(availWidth / totalWidth, availHeight / (maxHeight + FRAME_CHROME_HEIGHT));

  return Number(Math.max(minScale, Math.min(1, ratio)).toFixed(4));
});

watch(scale, (value) => {
  deviceManager.stageScale.value = value;
}, { immediate: true });
</script>

<template>
  <section class="device-stage" ref="stageRef">
    <div v-if="views.length === 0" class="empty-state">
      <div class="empty-icon">▤</div>
      <h2>还没有任何调试窗口</h2>
      <p>新建一个无痕窗口后，它会自动横向排列在初始窗口右侧，并按可用空间自动缩放。</p>
      <button class="empty-btn" @click="deviceManager.createIncognito()">+ 新建无痕窗口</button>
    </div>

    <div
      v-else
      class="stage-track"
      :class="layout"
      :style="{ padding: `${PADDING}px`, gap: `${GAP}px` }"
    >
      <DeviceFrame
        v-for="(view, index) in views"
        :key="view.id"
        :view="view"
        :index="index + 1"
        :scale="scale"
        :active="view.id === activeId"
        @activate="deviceManager.setActive(view.id)"
        @update="(patch) => onUpdate(view.id, patch)"
        @close="deviceManager.closeView(view.id)"
      />
    </div>
  </section>
</template>

<style scoped>
.device-stage {
  position: relative;
  flex: 1;
  min-height: 0;
  overflow: auto;
  background-color: var(--bg-0);
  background-image:
    linear-gradient(rgba(120, 150, 200, 0.045) 1px, transparent 1px),
    linear-gradient(90deg, rgba(120, 150, 200, 0.045) 1px, transparent 1px),
    radial-gradient(circle at 18% 8%, rgba(76, 141, 255, 0.1), transparent 42%),
    radial-gradient(circle at 84% 88%, rgba(124, 92, 255, 0.12), transparent 42%);
  background-size: 28px 28px, 28px 28px, 100% 100%, 100% 100%;
}

.stage-track {
  display: flex;
  align-items: safe center;
  min-width: min-content;
  min-height: 100%;
}

.stage-track.row {
  flex-wrap: nowrap;
  justify-content: safe center;
}

.stage-track.wrap {
  flex-wrap: wrap;
  align-content: flex-start;
  justify-content: safe center;
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 10px;
  height: 100%;
  padding: 40px;
  text-align: center;
  color: var(--fg-muted);
}

.empty-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 64px;
  height: 64px;
  border: 1px solid var(--border);
  border-radius: 18px;
  background: var(--bg-2);
  font-size: 26px;
  color: var(--accent);
}

.empty-state h2 {
  margin: 4px 0 0;
  font-size: 16px;
  color: var(--fg);
}

.empty-state p {
  max-width: 420px;
  margin: 0;
  font-size: 12px;
  line-height: 1.7;
}

.empty-btn {
  margin-top: 6px;
  padding: 8px 16px;
  border: none;
  border-radius: 9px;
  background: linear-gradient(135deg, var(--accent-2), #a06bff);
  color: #fff;
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
}
</style>
