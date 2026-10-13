import { computed, ref } from "vue";
import type { Workspace } from "../types";
import { deviceManager } from "./useDeviceManager";

const STORAGE_KEY = "mmwd.workspaces.v1";

function load(): Workspace[] {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return [];
    const parsed: unknown = JSON.parse(raw);
    if (!Array.isArray(parsed)) return [];
    return parsed.filter(
      (item): item is Workspace =>
        typeof item === "object" &&
        item !== null &&
        typeof (item as Workspace).id === "string" &&
        typeof (item as Workspace).name === "string" &&
        Array.isArray((item as Workspace).windows),
    );
  } catch {
    return [];
  }
}

function createWorkspaceManager() {
  const workspaces = ref<Workspace[]>(load());
  const currentId = ref<string | null>(null);

  /** 最近保存的排在最前 */
  const list = computed(() => [...workspaces.value].sort((a, b) => b.savedAt - a.savedAt));
  const current = computed(() => workspaces.value.find((item) => item.id === currentId.value) ?? null);

  function persist() {
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(workspaces.value));
    } catch {
      // 存储不可用时静默忽略，仅本次会话生效
    }
  }

  /** 以指定名称保存当前所有窗口；同名工作区直接覆盖 */
  function saveAs(name: string): Workspace | null {
    const trimmed = name.trim();
    if (!trimmed) return null;

    const existing = workspaces.value.find((item) => item.name === trimmed);
    const target: Workspace = existing ?? {
      id: `ws-${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 6)}`,
      name: trimmed,
      savedAt: 0,
      windows: [],
    };

    target.windows = deviceManager.snapshot();
    target.savedAt = Date.now();
    if (!existing) workspaces.value.push(target);

    currentId.value = target.id;
    persist();
    return target;
  }

  /** 覆盖更新当前打开的工作区 */
  function updateCurrent(): Workspace | null {
    const target = current.value;
    if (!target) return null;
    target.windows = deviceManager.snapshot();
    target.savedAt = Date.now();
    persist();
    return target;
  }

  function remove(id: string) {
    workspaces.value = workspaces.value.filter((item) => item.id !== id);
    if (currentId.value === id) currentId.value = null;
    persist();
  }

  /** 打开工作区：用其快照替换当前全部窗口 */
  function open(id: string) {
    const target = workspaces.value.find((item) => item.id === id);
    if (!target) return;
    deviceManager.restore(target.windows);
    currentId.value = target.id;
  }

  /** 新建工作区：清空窗口布局并解除与已保存工作区的关联 */
  function startNew() {
    currentId.value = null;
    deviceManager.resetToInitial();
  }

  return { list, current, currentId, saveAs, updateCurrent, remove, open, startNew };
}

export const workspaceManager = createWorkspaceManager();
