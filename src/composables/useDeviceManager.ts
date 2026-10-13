import { computed, ref, watch } from "vue";
import type { DeviceView, Orientation } from "../types";
import { DEFAULT_DEVICE_ID } from "../constants/devices";

export type LayoutMode = "row" | "wrap";

export const DEFAULT_URL = "https://example.com";

/**
 * 将用户输入补全为可导航的 URL。
 */
export function normalizeUrl(input: string): string {
  const raw = input.trim();
  if (!raw) return "";
  if (/^[a-z][a-z0-9+.-]*:\/\//i.test(raw)) return raw;
  if (/^(about|data|file|blob):/i.test(raw)) return raw;
  if (raw.startsWith("//")) return `https:${raw}`;
  if (/^localhost(:|\/|$)/i.test(raw) || /^127\.0\.0\.1(:|\/|$)/.test(raw) || /:\d{2,5}$/.test(raw)) {
    return `http://${raw}`;
  }
  return `https://${raw}`;
}

function createDeviceManager() {
  const views = ref<DeviceView[]>([]);
  const activeId = ref<string>("");
  const layout = ref<LayoutMode>("row");
  const fitToStage = ref(false);
  const address = ref(DEFAULT_URL);
  /** 由舞台计算并回写，用于状态栏展示当前缩放比例 */
  const stageScale = ref(1);

  let sequence = 0;

  const activeView = computed<DeviceView | null>(
    () => views.value.find((view) => view.id === activeId.value) ?? views.value[0] ?? null,
  );

  const incognitoCount = computed(() => views.value.filter((view) => view.incognito).length);

  watch(
    () => activeView.value?.url,
    (url) => {
      if (url !== undefined) address.value = url;
    },
    { immediate: true },
  );

  function setActive(id: string) {
    const target = views.value.find((view) => view.id === id);
    if (!target) return;
    activeId.value = id;
    address.value = target.url;
  }

  function patch(id: string, changes: Partial<DeviceView>) {
    const target = views.value.find((view) => view.id === id);
    if (!target) return;
    Object.assign(target, changes);
  }

  function createView(options?: {
    incognito?: boolean;
    url?: string;
    deviceId?: string;
    orientation?: Orientation;
    label?: string;
  }): DeviceView {
    sequence += 1;
    const incognito = options?.incognito ?? false;
    const url = normalizeUrl(options?.url ?? address.value ?? DEFAULT_URL);
    const view: DeviceView = {
      id: `view-${sequence}`,
      label: options?.label ?? (incognito ? `无痕窗口 ${incognitoCount.value + 1}` : "主窗口"),
      incognito,
      url,
      deviceId: options?.deviceId ?? activeView.value?.deviceId ?? DEFAULT_DEVICE_ID,
      orientation: options?.orientation ?? "portrait",
      reloadToken: 0,
      history: [url],
      historyIndex: 0,
    };
    views.value.push(view);
    activeId.value = view.id;
    address.value = view.url;
    return view;
  }

  function createIncognito() {
    return createView({ incognito: true });
  }

  function createWindow() {
    return createView({ incognito: false, label: `普通窗口 ${views.value.length + 1}` });
  }

  function closeView(id: string) {
    const index = views.value.findIndex((view) => view.id === id);
    if (index === -1) return;
    views.value.splice(index, 1);
    if (activeId.value === id) {
      const next = views.value[index] ?? views.value[index - 1] ?? null;
      activeId.value = next?.id ?? "";
      address.value = next?.url ?? DEFAULT_URL;
    }
  }

  function closeActive() {
    if (activeView.value) closeView(activeView.value.id);
  }

  function closeAllIncognito() {
    views.value
      .filter((view) => view.incognito)
      .map((view) => view.id)
      .forEach((id) => closeView(id));
  }

  function jumpToHistory(view: DeviceView, index: number) {
    const next = view.history[index];
    if (!next) return;
    view.historyIndex = index;
    view.url = next;
    view.reloadToken += 1;
  }

  function navigate(targetUrl: string, options?: { applyAll?: boolean; viewId?: string }) {
    const url = normalizeUrl(targetUrl);
    if (!url) return;
    const targets = options?.applyAll
      ? views.value
      : [options?.viewId ? views.value.find((view) => view.id === options.viewId) : activeView.value].filter(
          (view): view is DeviceView => Boolean(view),
        );

    for (const view of targets) {
      if (view.url === url) {
        view.reloadToken += 1;
        continue;
      }
      view.history = [...view.history.slice(0, view.historyIndex + 1), url];
      view.historyIndex = view.history.length - 1;
      view.url = url;
      view.reloadToken += 1;
      if (view.id === activeId.value) address.value = url;
    }
  }

  function canGoBack(view: DeviceView): boolean {
    return view.historyIndex > 0;
  }

  function canGoForward(view: DeviceView): boolean {
    return view.historyIndex < view.history.length - 1;
  }

  function goBack(viewId?: string) {
    const target = viewId ? views.value.find((view) => view.id === viewId) : activeView.value;
    if (target && canGoBack(target)) jumpToHistory(target, target.historyIndex - 1);
  }

  function goForward(viewId?: string) {
    const target = viewId ? views.value.find((view) => view.id === viewId) : activeView.value;
    if (target && canGoForward(target)) jumpToHistory(target, target.historyIndex + 1);
  }

  function reload(id?: string) {
    const target = id ? views.value.find((view) => view.id === id) : activeView.value;
    if (target) target.reloadToken += 1;
  }

  function reloadAll() {
    views.value.forEach((view) => {
      view.reloadToken += 1;
    });
  }

  function setOrientation(id: string, orientation: Orientation) {
    patch(id, { orientation });
  }

  function resetScale() {
    fitToStage.value = false;
    layout.value = "row";
  }

  function toggleLayout() {
    layout.value = layout.value === "row" ? "wrap" : "row";
  }

  function toggleFit() {
    fitToStage.value = !fitToStage.value;
  }

  return {
    views,
    activeId,
    activeView,
    layout,
    fitToStage,
    address,
    stageScale,
    incognitoCount,
    setActive,
    patch,
    createView,
    createIncognito,
    createWindow,
    closeView,
    closeActive,
    closeAllIncognito,
    navigate,
    goBack,
    goForward,
    canGoBack,
    canGoForward,
    reload,
    reloadAll,
    setOrientation,
    resetScale,
    toggleLayout,
    toggleFit,
  };
}

export const deviceManager = createDeviceManager();
