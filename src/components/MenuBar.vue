<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from "vue";
import { deviceManager } from "../composables/useDeviceManager";
import { workspaceManager } from "../composables/useWorkspaces";
import { findDevice } from "../constants/devices";

const emit = defineEmits<{ (e: "open-about"): void }>();

type MenuId = "file" | "view" | "window" | "help";

type MenuItem =
  | {
      kind: "item";
      label: string;
      shortcut?: string;
      hint?: string;
      disabled?: boolean;
      checked?: boolean;
      action: () => void;
    }
  | { kind: "separator" };

const menuRoot = ref<HTMLElement | null>(null);
const openMenuId = ref<MenuId | null>(null);

const viewCount = computed(() => deviceManager.views.value.length);
const incognitoCount = computed(() => deviceManager.incognitoCount.value);

const workspaceMenuOpen = ref(false);
const namingOpen = ref(false);
const workspaceName = ref("");
const notice = ref("");
const nameInput = ref<HTMLInputElement | null>(null);
let noticeTimer: number | undefined;

watch(namingOpen, (open) => {
  if (open) void nextTick(() => nameInput.value?.focus());
});

const workspaceList = computed(() => workspaceManager.list.value);
const currentWorkspace = computed(() => workspaceManager.current.value);
const saveButtonLabel = computed(() => (currentWorkspace.value ? "更新工作区" : "保存工作区"));

function flash(message: string) {
  notice.value = message;
  if (noticeTimer !== undefined) window.clearTimeout(noticeTimer);
  noticeTimer = window.setTimeout(() => {
    notice.value = "";
  }, 2600);
}

function toggleWorkspaceMenu() {
  workspaceMenuOpen.value = !workspaceMenuOpen.value;
  openMenuId.value = null;
  namingOpen.value = false;
}

function openWorkspace(id: string, name: string) {
  workspaceManager.open(id);
  workspaceMenuOpen.value = false;
  flash(`已载入「${name}」`);
}

function removeWorkspace(id: string, name: string) {
  workspaceManager.remove(id);
  flash(`已删除「${name}」`);
}

function handleSaveWorkspace() {
  workspaceMenuOpen.value = false;
  const opened = currentWorkspace.value;
  if (opened) {
    workspaceManager.updateCurrent();
    flash(`已更新「${opened.name}」`);
    return;
  }
  workspaceName.value = "";
  namingOpen.value = true;
}

function confirmSaveWorkspace() {
  const created = workspaceManager.saveAs(workspaceName.value);
  if (!created) return;
  namingOpen.value = false;
  flash(`已保存「${created.name}」`);
}

function startNewWorkspace() {
  workspaceMenuOpen.value = false;
  namingOpen.value = false;
  workspaceManager.startNew();
  flash("已新建工作区");
}

function formatSavedAt(timestamp: number) {
  const date = new Date(timestamp);
  const pad = (value: number) => String(value).padStart(2, "0");
  return `${pad(date.getMonth() + 1)}-${pad(date.getDate())} ${pad(date.getHours())}:${pad(date.getMinutes())}`;
}

function toggleMenu(id: MenuId) {
  openMenuId.value = openMenuId.value === id ? null : id;
}

function run(action: () => void) {
  openMenuId.value = null;
  action();
}

function onDocumentPointerDown(event: MouseEvent) {
  if (menuRoot.value && !menuRoot.value.contains(event.target as Node)) {
    openMenuId.value = null;
    workspaceMenuOpen.value = false;
    namingOpen.value = false;
  }
}

onMounted(() => document.addEventListener("mousedown", onDocumentPointerDown));
onBeforeUnmount(() => {
  document.removeEventListener("mousedown", onDocumentPointerDown);
  if (noticeTimer !== undefined) window.clearTimeout(noticeTimer);
});

async function quitApp() {
  try {
    const { getCurrentWebviewWindow } = await import("@tauri-apps/api/webviewWindow");
    await getCurrentWebviewWindow().close();
  } catch {
    window.close();
  }
}

function sizeLabel(deviceId: string, orientation: string): string {
  const device = findDevice(deviceId);
  return orientation === "landscape" ? `${device.height}×${device.width}` : `${device.width}×${device.height}`;
}

const menus = computed<{ id: MenuId; label: string; items: MenuItem[] }[]>(() => {
  const active = deviceManager.activeView.value;
  const windowItems: MenuItem[] = deviceManager.views.value.length
    ? deviceManager.views.value.map((view) => ({
        kind: "item",
        label: view.label,
        hint: sizeLabel(view.deviceId, view.orientation),
        checked: view.id === deviceManager.activeId.value,
        action: () => deviceManager.setActive(view.id),
      }))
    : [{ kind: "item", label: "暂无窗口", disabled: true, action: () => undefined }];

  return [
    {
      id: "file",
      label: "文件",
      items: [
        { kind: "item", label: "新建无痕窗口", shortcut: "Ctrl+Shift+N", action: deviceManager.createIncognito },
        { kind: "item", label: "新建窗口", shortcut: "Ctrl+N", action: deviceManager.createWindow },
        { kind: "separator" },
        {
          kind: "item",
          label: "关闭当前窗口",
          shortcut: "Ctrl+W",
          disabled: !active,
          action: deviceManager.closeActive,
        },
        {
          kind: "item",
          label: "关闭全部无痕窗口",
          disabled: deviceManager.incognitoCount.value === 0,
          action: deviceManager.closeAllIncognito,
        },
        { kind: "separator" },
        { kind: "item", label: "退出", action: quitApp },
      ],
    },
    {
      id: "view",
      label: "视图",
      items: [
        {
          kind: "item",
          label: "刷新当前窗口",
          shortcut: "Ctrl+R",
          disabled: !active,
          action: () => deviceManager.reload(),
        },
        { kind: "item", label: "刷新全部窗口", disabled: !active, action: deviceManager.reloadAll },
        { kind: "separator" },
        {
          kind: "item",
          label: "自动缩放以适应宽度",
          checked: deviceManager.fitToStage.value,
          action: deviceManager.toggleFit,
        },
        {
          kind: "item",
          label: "横向排列（单行）",
          checked: deviceManager.layout.value === "row",
          action: deviceManager.toggleLayout,
        },
        { kind: "item", label: "重置舞台", action: deviceManager.resetScale },
      ],
    },
    { id: "window", label: "窗口", items: windowItems },
    {
      id: "help",
      label: "帮助",
      items: [{ kind: "item", label: "关于 MobileMultiWindowDebugger", action: () => emit("open-about") }],
    },
  ];
});
</script>

<template>
  <header class="menu-bar" ref="menuRoot">
    <nav class="menus">
      <div v-for="menu in menus" :key="menu.id" class="menu">
        <button class="menu-title" :class="{ open: openMenuId === menu.id }" @click="toggleMenu(menu.id)">
          {{ menu.label }}
        </button>
        <div v-if="openMenuId === menu.id" class="menu-dropdown">
          <template v-for="(item, index) in menu.items" :key="`${menu.id}-${index}`">
            <div v-if="item.kind === 'separator'" class="menu-separator" />
            <button
              v-else
              class="menu-item"
              :class="{ disabled: item.disabled }"
              :disabled="item.disabled"
              @click="run(item.action)"
            >
              <span class="menu-item-check">{{ item.checked ? "✓" : "" }}</span>
              <span class="menu-item-label">{{ item.label }}</span>
              <span v-if="item.hint" class="menu-item-hint">{{ item.hint }}</span>
              <span v-else class="menu-item-shortcut">{{ item.shortcut }}</span>
            </button>
          </template>
        </div>
      </div>
    </nav>

    <div class="menu-bar-right">
      <span v-if="notice" class="status-chip success">{{ notice }}</span>
      <span class="status-chip">{{ viewCount }} 个窗口</span>
      <span v-if="incognitoCount > 0" class="status-chip incognito">无痕 {{ incognitoCount }}</span>

      <div class="workspace">
        <button class="ws-btn" :class="{ open: workspaceMenuOpen }" title="选择 / 管理已保存的工作区" @click="toggleWorkspaceMenu">
          <span class="ws-btn-name">{{ currentWorkspace ? currentWorkspace.name : "工作区" }}</span>
          <span class="caret">▾</span>
        </button>

        <div v-if="workspaceMenuOpen" class="ws-panel">
          <div class="ws-panel-title">已保存的工作区</div>
          <div v-if="workspaceList.length === 0" class="ws-empty">暂无工作区，点击「保存工作区」创建</div>
          <div
            v-for="item in workspaceList"
            :key="item.id"
            class="ws-row"
            :class="{ active: item.id === currentWorkspace?.id }"
            @click="openWorkspace(item.id, item.name)"
          >
            <span class="ws-name">{{ item.name }}</span>
            <span class="ws-meta">{{ item.windows.length }} 个窗口 · {{ formatSavedAt(item.savedAt) }}</span>
            <button class="ws-remove" title="删除工作区" @click.stop="removeWorkspace(item.id, item.name)">✕</button>
          </div>
        </div>

        <button class="ws-btn primary" :title="saveButtonLabel + '（保存当前全部窗口）'" @click="handleSaveWorkspace">
          {{ saveButtonLabel }}
        </button>
        <button class="ws-btn" title="清空窗口布局并开始一个新工作区" @click="startNewWorkspace">新建工作区</button>

        <div v-if="namingOpen" class="ws-name-dialog">
          <div class="ws-name-title">保存当前工作区（{{ viewCount }} 个窗口）</div>
          <input
            ref="nameInput"
            v-model.trim="workspaceName"
            class="ws-name-input"
            placeholder="输入工作区名称"
            @keydown.enter="confirmSaveWorkspace"
            @keydown.esc="namingOpen = false"
          />
          <div class="ws-name-actions">
            <button class="ws-btn" @click="namingOpen = false">取消</button>
            <button class="ws-btn primary" :disabled="!workspaceName" @click="confirmSaveWorkspace">保存</button>
          </div>
        </div>
      </div>
    </div>
  </header>
</template>

<style scoped>
.menu-bar {
  display: flex;
  align-items: center;
  height: 38px;
  padding: 0 8px;
  background: var(--bg-1);
  border-bottom: 1px solid var(--border);
  user-select: none;
}

.menus {
  display: flex;
  align-items: center;
  gap: 2px;
}

.menu {
  position: relative;
}

.menu-title {
  height: 26px;
  padding: 0 12px;
  border: none;
  border-radius: 6px;
  background: transparent;
  color: var(--fg-dim);
  font-size: 13px;
  cursor: pointer;
}

.menu-title:hover,
.menu-title.open {
  background: var(--bg-3);
  color: var(--fg);
}

.menu-dropdown {
  position: absolute;
  z-index: 40;
  top: 30px;
  left: 0;
  min-width: 232px;
  padding: 6px;
  background: var(--bg-3);
  border: 1px solid var(--border);
  border-radius: 10px;
  box-shadow: 0 18px 40px rgba(0, 0, 0, 0.45);
}

.menu-item {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  padding: 7px 10px;
  border: none;
  border-radius: 7px;
  background: transparent;
  color: var(--fg);
  font-size: 13px;
  text-align: left;
  cursor: pointer;
}

.menu-item:hover:not(.disabled) {
  background: var(--bg-hover);
}

.menu-item.disabled {
  color: var(--fg-muted);
  cursor: default;
}

.menu-item-check {
  width: 12px;
  color: var(--accent);
}

.menu-item-label {
  flex: 1;
  white-space: nowrap;
}

.menu-item-shortcut,
.menu-item-hint {
  color: var(--fg-muted);
  font-size: 11px;
}

.menu-separator {
  height: 1px;
  margin: 5px 6px;
  background: var(--border);
}

.menu-bar-right {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-left: auto;
}

.status-chip {
  padding: 3px 9px;
  border: 1px solid var(--border);
  border-radius: 999px;
  color: var(--fg-dim);
  font-size: 11px;
}

.status-chip.incognito {
  border-color: rgba(124, 92, 255, 0.5);
  background: rgba(124, 92, 255, 0.14);
  color: #c3b4ff;
}

.status-chip.success {
  border-color: rgba(61, 220, 151, 0.45);
  background: rgba(61, 220, 151, 0.14);
  color: #8ef0c0;
}

.workspace {
  position: relative;
  display: flex;
  align-items: center;
  gap: 6px;
  margin-left: 4px;
  padding-left: 10px;
  border-left: 1px solid var(--border);
}

.ws-btn {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  height: 24px;
  padding: 0 10px;
  border: 1px solid var(--border);
  border-radius: 7px;
  background: var(--bg-3);
  color: var(--fg-dim);
  font-size: 11.5px;
  white-space: nowrap;
  cursor: pointer;
  transition: background 0.15s, color 0.15s, border-color 0.15s;
}

.ws-btn:hover:not(:disabled) {
  background: var(--bg-hover);
  color: var(--fg);
}

.ws-btn.open {
  border-color: rgba(76, 141, 255, 0.55);
  background: rgba(76, 141, 255, 0.16);
  color: #b9d2ff;
}

.ws-btn.primary {
  background: var(--accent);
  border-color: transparent;
  color: #fff;
  font-weight: 600;
}

.ws-btn.primary:hover:not(:disabled) {
  background: #6aa1ff;
  color: #fff;
}

.ws-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.ws-btn-name {
  max-width: 148px;
  overflow: hidden;
  white-space: nowrap;
  text-overflow: ellipsis;
}

.caret {
  font-size: 9px;
  opacity: 0.8;
}

.ws-panel {
  position: absolute;
  z-index: 60;
  top: 28px;
  right: 0;
  width: 288px;
  max-height: 320px;
  overflow: auto;
  padding: 6px;
  border: 1px solid var(--border);
  border-radius: 10px;
  background: var(--bg-3);
  box-shadow: 0 18px 40px rgba(0, 0, 0, 0.45);
}

.ws-panel-title {
  padding: 4px 8px 6px;
  color: var(--fg-muted);
  font-size: 11px;
}

.ws-empty {
  padding: 10px 8px 12px;
  color: var(--fg-muted);
  font-size: 12px;
  line-height: 1.6;
}

.ws-row {
  position: relative;
  display: flex;
  flex-direction: column;
  gap: 2px;
  padding: 6px 28px 6px 8px;
  border-radius: 7px;
  cursor: pointer;
}

.ws-row:hover {
  background: var(--bg-hover);
}

.ws-row.active {
  background: rgba(76, 141, 255, 0.14);
  box-shadow: inset 0 0 0 1px rgba(76, 141, 255, 0.35);
}

.ws-name {
  overflow: hidden;
  color: var(--fg);
  font-size: 12.5px;
  white-space: nowrap;
  text-overflow: ellipsis;
}

.ws-meta {
  color: var(--fg-muted);
  font-size: 10.5px;
}

.ws-remove {
  position: absolute;
  top: 6px;
  right: 6px;
  display: grid;
  place-items: center;
  width: 18px;
  height: 18px;
  border: none;
  border-radius: 5px;
  background: transparent;
  color: var(--fg-muted);
  font-size: 11px;
  cursor: pointer;
}

.ws-remove:hover {
  background: rgba(255, 92, 108, 0.18);
  color: #ff8b90;
}

.ws-name-dialog {
  position: absolute;
  z-index: 70;
  top: 28px;
  right: 0;
  width: 250px;
  padding: 10px;
  border: 1px solid var(--border);
  border-radius: 10px;
  background: var(--bg-3);
  box-shadow: 0 18px 40px rgba(0, 0, 0, 0.45);
}

.ws-name-title {
  margin-bottom: 8px;
  color: var(--fg-dim);
  font-size: 12px;
}

.ws-name-input {
  width: 100%;
  height: 28px;
  padding: 0 8px;
  border: 1px solid var(--border);
  border-radius: 7px;
  background: var(--bg-0);
  color: var(--fg);
  font-size: 12px;
  outline: none;
}

.ws-name-input:focus {
  border-color: rgba(76, 141, 255, 0.6);
  box-shadow: 0 0 0 3px rgba(76, 141, 255, 0.14);
}

.ws-name-actions {
  display: flex;
  justify-content: flex-end;
  gap: 6px;
  margin-top: 9px;
}
</style>
