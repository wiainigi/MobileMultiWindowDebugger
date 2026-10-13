<script setup lang="ts">
import { computed, ref } from "vue";
import { deviceManager } from "../composables/useDeviceManager";

const activeView = computed(() => deviceManager.activeView.value);
const canBack = computed(() => (activeView.value ? deviceManager.canGoBack(activeView.value) : false));
const canForward = computed(() => (activeView.value ? deviceManager.canGoForward(activeView.value) : false));
const syncAll = ref(false);

const address = computed<string>({
  get: () => deviceManager.address.value,
  set: (value: string) => {
    deviceManager.address.value = value;
  },
});

const fitToStage = computed(() => deviceManager.fitToStage.value);
const layout = computed(() => deviceManager.layout.value);

function submit() {
  deviceManager.navigate(address.value, { applyAll: syncAll.value });
}

</script>

<template>
  <div class="toolbar">
    <div class="nav-group">
      <button class="tool-btn" title="后退" :disabled="!canBack" @click="deviceManager.goBack()">←</button>
      <button class="tool-btn" title="前进" :disabled="!canForward" @click="deviceManager.goForward()">→</button>
      <button class="tool-btn" title="刷新当前窗口" :disabled="!activeView" @click="deviceManager.reload()">⟳</button>
    </div>

    <form class="address-bar" @submit.prevent="submit">
      <span class="scheme-tag" :class="{ incognito: activeView?.incognito }">
        {{ activeView?.incognito ? "无痕" : "HTTPS" }}
      </span>
      <input
        v-model.trim="address"
        class="address-input"
        type="text"
        spellcheck="false"
        autocomplete="off"
        placeholder="输入网址后回车，例如 m.example.com"
      />
      <label class="sync-toggle" title="将地址同步到全部窗口">
        <input v-model="syncAll" type="checkbox" />
        <span>同步全部</span>
      </label>
      <button class="go-btn" type="submit" :disabled="!activeView" title="前往">→</button>
    </form>

    <div class="tool-group">
      <button class="tool-btn" :disabled="!activeView" title="刷新所有窗口" @click="deviceManager.reloadAll()">
        全部重载
      </button>
    </div>

    <div class="divider" />

    <div class="window-tools">
      <button
        class="tool-btn"
        :class="{ active: fitToStage }"
        title="自动缩放以适应舞台宽度"
        @click="deviceManager.toggleFit()"
      >
        适应宽度
      </button>
      <button
        class="tool-btn"
        :title="layout === 'row' ? '切换为自动换行' : '切换为单行横向排列'"
        @click="deviceManager.toggleLayout()"
      >
        {{ layout === "row" ? "单行排列" : "自动换行" }}
      </button>

      <div class="divider" />

      <button class="primary-btn accent" title="新建一个无痕窗口（Ctrl+Shift+N）" @click="deviceManager.createIncognito()">
        + 新建无痕窗口
      </button>
      <button class="tool-btn" title="新建普通窗口（Ctrl+N）" @click="deviceManager.createWindow()">+ 窗口</button>
      <button class="tool-btn danger" :disabled="!activeView" title="关闭当前窗口（Ctrl+W）" @click="deviceManager.closeActive()">
        关闭
      </button>
    </div>
  </div>
</template>

<style scoped>
.toolbar {
  display: flex;
  align-items: center;
  gap: 10px;
  min-height: 46px;
  padding: 8px 12px;
  background: var(--bg-2);
  border-bottom: 1px solid var(--border);
}

.nav-group,
.tool-group,
.window-tools {
  display: flex;
  align-items: center;
  gap: 6px;
}

.tool-btn,
.primary-btn {
  display: inline-flex;
  align-items: center;
  height: 30px;
  padding: 0 11px;
  border: 1px solid var(--border);
  border-radius: 8px;
  background: var(--bg-3);
  color: var(--fg-dim);
  font-size: 12px;
  white-space: nowrap;
  cursor: pointer;
  transition: background 0.15s, color 0.15s, border-color 0.15s;
}

.tool-btn:hover:not(:disabled),
.primary-btn:hover:not(:disabled) {
  background: var(--bg-hover);
  color: var(--fg);
}

.tool-btn:disabled,
.primary-btn:disabled {
  opacity: 0.45;
  cursor: not-allowed;
}

.tool-btn.active {
  border-color: rgba(76, 141, 255, 0.55);
  background: rgba(76, 141, 255, 0.16);
  color: #b9d2ff;
}

.primary-btn {
  background: var(--accent);
  border-color: transparent;
  color: #fff;
  font-weight: 600;
}

.primary-btn:hover:not(:disabled) {
  background: #6aa1ff;
  color: #fff;
}

.primary-btn.accent {
  background: linear-gradient(135deg, var(--accent-2), #a06bff);
  box-shadow: 0 6px 16px rgba(124, 92, 255, 0.28);
}

.danger:hover:not(:disabled) {
  border-color: rgba(255, 92, 108, 0.6);
  color: #ff8b90;
}

.address-bar {
  display: flex;
  align-items: center;
  flex: 1;
  gap: 8px;
  min-width: 120px;
  height: 32px;
  padding: 0 3px 0 10px;
  border: 1px solid var(--border);
  border-radius: 999px;
  background: var(--bg-0);
}

.go-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex: none;
  min-width: 36px;
  height: 26px;
  padding: 0 10px;
  border: none;
  border-radius: 999px;
  background: var(--accent);
  color: #fff;
  font-size: 13px;
  font-weight: 700;
  line-height: 1;
  cursor: pointer;
  transition: background 0.15s;
}

.go-btn:hover:not(:disabled) {
  background: #6aa1ff;
}

.go-btn:disabled {
  opacity: 0.45;
  cursor: not-allowed;
}

.address-bar:focus-within {
  border-color: rgba(76, 141, 255, 0.6);
  box-shadow: 0 0 0 3px rgba(76, 141, 255, 0.16);
}

.scheme-tag {
  padding: 2px 8px;
  border-radius: 999px;
  background: var(--bg-3);
  color: var(--fg-muted);
  font-size: 11px;
  white-space: nowrap;
}

.scheme-tag.incognito {
  background: rgba(124, 92, 255, 0.18);
  color: #c3b4ff;
}

.address-input {
  flex: 1;
  min-width: 0;
  border: none;
  background: transparent;
  color: var(--fg);
  font-size: 13px;
  outline: none;
}

.sync-toggle {
  display: flex;
  align-items: center;
  gap: 4px;
  color: var(--fg-muted);
  font-size: 11px;
  white-space: nowrap;
  cursor: pointer;
}

.sync-toggle input {
  accent-color: var(--accent);
}

.divider {
  width: 1px;
  height: 22px;
  background: var(--border);
}
</style>
