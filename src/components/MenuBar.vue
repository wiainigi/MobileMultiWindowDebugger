<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from "vue";
import { deviceManager } from "../composables/useDeviceManager";
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
  }
}

onMounted(() => document.addEventListener("mousedown", onDocumentPointerDown));
onBeforeUnmount(() => document.removeEventListener("mousedown", onDocumentPointerDown));

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
      <span class="status-chip">{{ viewCount }} 个窗口</span>
      <span v-if="incognitoCount > 0" class="status-chip incognito">无痕 {{ incognitoCount }}</span>
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
</style>
