<script setup lang="ts">
import { computed, onBeforeUnmount, ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { DeviceView } from "../types";
import { DEFAULT_URL } from "../composables/useDeviceManager";
import { DEVICE_GROUPS, findDevice } from "../constants/devices";

const props = defineProps<{
  view: DeviceView;
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
 * 统一保留 allow-same-origin，页面才能正常读写自身存储；
 * 无痕窗口的存储隔离改由「每窗口独立端口的本地代理」实现（见 renderUrl）。
 */
const SANDBOX = [
  "allow-scripts",
  "allow-same-origin",
  "allow-forms",
  "allow-popups",
  "allow-popups-to-escape-sandbox",
  "allow-modals",
  "allow-downloads",
].join(" ");

const proxyPort = ref<number | null>(null);
const proxyError = ref("");
let proxyRequestId = 0;

/** 无痕窗口需要被代理的目标 origin；普通窗口返回空串表示直连 */
const targetOrigin = computed(() => {
  if (!props.view.incognito) return "";
  try {
    const parsed = new URL(props.view.url);
    if (parsed.protocol !== "http:" && parsed.protocol !== "https:") return "";
    return `${parsed.protocol}//${parsed.host}`;
  } catch {
    return "";
  }
});

/**
 * 实际加载地址：
 * 普通窗口直连目标站点；无痕窗口经由本地独立端口代理，
 * 页面 origin 变为 http://127.0.0.1:{port}，与其它所有窗口互不共享存储。
 */
const renderUrl = computed(() => {
  if (!props.view.incognito) return props.view.url;
  // 内置相对页面无需代理，直接加载
  if (!targetOrigin.value) return props.view.url;
  if (proxyPort.value === null) return "";
  try {
    const parsed = new URL(props.view.url);
    return `http://127.0.0.1:${proxyPort.value}${parsed.pathname}${parsed.search}${parsed.hash}`;
  } catch {
    return "";
  }
});

async function stopProxy() {
  const port = proxyPort.value;
  proxyPort.value = null;
  if (port === null) return;
  try {
    await invoke("stop_window_proxy", { port });
  } catch {
    // 代理已随进程释放，忽略
  }
}

watch(
  targetOrigin,
  async (origin) => {
    const requestId = (proxyRequestId += 1);
    if (!origin) {
      await stopProxy();
      proxyError.value = "";
      return;
    }
    try {
      const port = await invoke<number>("start_window_proxy", { origin });
      if (requestId !== proxyRequestId) {
        // 期间地址已变更，丢弃这次启动的代理
        await invoke("stop_window_proxy", { port }).catch(() => undefined);
        return;
      }
      await stopProxy();
      proxyPort.value = port;
      proxyError.value = "";
      debugLog(`proxy ready ${origin} -> http://127.0.0.1:${port}`);
    } catch (error) {
      proxyError.value = error instanceof Error ? error.message : String(error);
      debugLog(`proxy failed: ${proxyError.value}`);
    }
  },
  { immediate: true },
);

onBeforeUnmount(() => {
  void stopProxy();
});



const iframeRef = ref<HTMLIFrameElement | null>(null);
const documentTitle = ref("");
const titleError = ref("");
let titleRequestId = 0;

function debugLog(message: string) {
  void invoke("debug_log", { message }).catch(() => undefined);
}

/** 同源页面可直接读取运行时的 document.title（含 SPA 动态设置的标题） */
function readSameOriginTitle(): string {
  try {
    return iframeRef.value?.contentDocument?.title?.trim() ?? "";
  } catch {
    return "";
  }
}

/**
 * 地址变化或刷新时主动解析网页标题，不依赖 iframe 的 load 时序：
 * 同源页面直接读取 document.title；跨源页面受同源策略限制，
 * 交给 Rust 侧代理请求并解析 <title>。
 */
async function resolveTitle() {
  const url = props.view.url;
  documentTitle.value = "";
  titleError.value = "";

  const sameOrigin = readSameOriginTitle();
  if (sameOrigin) {
    debugLog(`same-origin title="${sameOrigin}"`);
    documentTitle.value = sameOrigin;
    return;
  }
  if (!/^https?:\/\//i.test(url)) return;

  const requestId = (titleRequestId += 1);
  debugLog(`request title url=${url} id=${requestId}`);
  try {
    const title = await invoke<string>("fetch_page_title", { url });
    debugLog(`invoke ok title="${title}" request=${requestId} current=${titleRequestId}`);
    if (requestId !== titleRequestId) return;
    if (title) {
      documentTitle.value = title;
      debugLog(`applied -> siteTitle="${siteTitle.value}"`);
    } else {
      titleError.value = "页面没有 <title>";
    }
  } catch (error) {
    titleError.value = error instanceof Error ? error.message : String(error);
    debugLog(`invoke failed: ${String(error)}`);
  }
}

/** iframe 加载完成后再读一次：同源页面的 SPA 运行时标题可能此时才确定 */
function onIframeLoad() {
  const sameOrigin = readSameOriginTitle();
  if (sameOrigin && sameOrigin !== documentTitle.value) {
    documentTitle.value = sameOrigin;
    titleError.value = "";
    debugLog(`load override title="${sameOrigin}"`);
  }
}

watch(
  () => [props.view.url, props.view.reloadToken] as const,
  () => {
    titleRequestId += 1;
    void resolveTitle();
  },
  { immediate: true },
);

const siteTitle = computed(() => {
  if (documentTitle.value) return documentTitle.value;
  const url = props.view.url;
  if (!url) return props.view.label;
  // 相对路径指向项目内置页面
  if (!/^[a-z][a-z0-9+.-]*:/i.test(url)) {
    return url === DEFAULT_URL ? "默认页" : url;
  }
  if (/^(about|data|blob|file):/i.test(url)) return props.view.label;
  try {
    const parsed = new URL(url);
    return parsed.port && parsed.port !== "80" && parsed.port !== "443"
      ? `${parsed.hostname}:${parsed.port}`
      : parsed.hostname;
  } catch {
    return props.view.label;
  }
});

function changeDevice(event: Event) {
  emit("update", { deviceId: (event.target as HTMLSelectElement).value });
}

function toggleOrientation() {
  emit("update", { orientation: props.view.orientation === "portrait" ? "landscape" : "portrait" });
}

function reload() {
  emit("update", { reloadToken: props.view.reloadToken + 1 });
}

const devtoolsOpen = ref(false);

/** 打开承载各调试窗口的 WebView 的开发者工具（再点一次关闭） */
async function toggleDevtools() {
  try {
    devtoolsOpen.value = await invoke<boolean>("toggle_devtools");
  } catch (error) {
    debugLog(`devtools failed: ${String(error)}`);
  }
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
      <div class="header-row header-row-top">
        <span class="frame-title" :title="`${siteTitle} — ${view.url}`">{{ siteTitle }}</span>
        <span v-if="titleError" class="title-warning" :title="`标题获取失败：${titleError}`">!</span>
        <span v-if="view.incognito" class="badge">无痕</span>
      </div>

      <div class="header-row header-row-bottom">
        <div class="header-left">
          <select
            class="frame-select"
            :value="view.deviceId"
            title="切换机型分辨率"
            @change="changeDevice"
            @mousedown.stop="emit('activate')"
          >
            <optgroup v-for="group in DEVICE_GROUPS" :key="group.name" :label="group.name">
              <option v-for="preset in group.devices" :key="preset.id" :value="preset.id">
                {{ preset.name }} · {{ preset.width }}×{{ preset.height }}
              </option>
            </optgroup>
          </select>
          <span class="size-tag">{{ logicalSize.width }}×{{ logicalSize.height }}</span>
        </div>

        <div class="header-right">
          <button class="icon-btn" title="旋转屏幕" @mousedown.stop="emit('activate')" @click="toggleOrientation">⟲</button>
          <button class="icon-btn" title="刷新" @mousedown.stop="emit('activate')" @click="reload">⟳</button>
          <button
            class="icon-btn console-btn"
            :class="{ on: devtoolsOpen }"
            :title="`开发者工具（在 Console 左上角的上下文选择器中切换到「${view.label}」frame；再次点击关闭）`"
            @mousedown.stop="emit('activate')"
            @click="toggleDevtools"
          >
            &gt;_
          </button>
          <button class="icon-btn" title="在默认浏览器中打开" @mousedown.stop="emit('activate')" @click="openExternal">↗</button>
          <button class="icon-btn danger" title="关闭" @mousedown.stop="emit('activate')" @click="emit('close')">✕</button>
        </div>
      </div>
    </header>

    <div class="frame-viewport" :style="viewportStyle">
      <div class="frame-screen" :style="screenStyle">
        <iframe
          v-if="renderUrl"
          ref="iframeRef"
          :key="`${view.id}-${view.reloadToken}-${renderUrl}`"
          class="frame-iframe"
          :name="view.label"
          :data-view-id="view.id"
          :src="renderUrl"
          :sandbox="SANDBOX"
          referrerpolicy="no-referrer-when-downgrade"
          allow="geolocation; camera; microphone; fullscreen; accelerometer; gyroscope; clipboard-write"
          @load="onIframeLoad"
        />
        <div v-else class="frame-placeholder">
          {{ proxyError ? `独立会话启动失败：${proxyError}` : "正在准备独立会话…" }}
        </div>
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
  flex-direction: column;
  gap: 5px;
  padding: 7px 9px;
  background: var(--frame-chrome);
  border-bottom: 1px solid var(--frame-border);
}

.active .frame-header {
  background: var(--frame-chrome-active);
}

.header-row {
  display: flex;
  align-items: center;
  gap: 8px;
  min-width: 0;
}

.header-row-top {
  height: 20px;
}

.frame-title {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  font-size: 12.5px;
  font-weight: 600;
  white-space: nowrap;
  text-overflow: ellipsis;
}

.title-warning {
  display: grid;
  place-items: center;
  flex: none;
  width: 14px;
  height: 14px;
  border-radius: 50%;
  background: rgba(255, 176, 76, 0.18);
  color: #ffb04c;
  font-size: 10px;
  font-weight: 700;
  cursor: help;
}

.badge {
  flex: none;
  padding: 1px 7px;
  border-radius: 999px;
  background: rgba(124, 92, 255, 0.24);
  color: #cbbcff;
  font-size: 10px;
  font-weight: 600;
}

.header-row-bottom {
  justify-content: space-between;
}

.header-left,
.header-right {
  display: flex;
  align-items: center;
  gap: 6px;
  min-width: 0;
}

.header-right {
  flex: none;
}

.frame-select {
  max-width: 118px;
  height: 22px;
  padding: 0 4px;
  border: 1px solid var(--frame-border);
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

.console-btn {
  width: auto;
  min-width: 22px;
  padding: 0 4px;
  font-family: "Cascadia Mono", "Consolas", monospace;
  font-size: 10.5px;
  font-weight: 700;
  letter-spacing: -0.5px;
}

.console-btn.on {
  background: rgba(76, 141, 255, 0.2);
  color: #b9d2ff;
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

.frame-placeholder {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 16px;
  background: var(--frame-screen);
  color: var(--fg-muted);
  font-size: 12px;
  line-height: 1.6;
  text-align: center;
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
  border: 1px solid var(--frame-border);
  border-radius: 999px;
  color: var(--fg-dim);
  font-variant-numeric: tabular-nums;
}
</style>
