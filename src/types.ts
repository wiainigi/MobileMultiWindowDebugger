export type Orientation = "portrait" | "landscape";

export interface DeviceProfile {
  id: string;
  name: string;
  group: string;
  /** CSS 逻辑宽度，单位 px */
  width: number;
  /** CSS 逻辑高度，单位 px */
  height: number;
  dpr: number;
}

export interface DeviceGroup {
  name: string;
  devices: DeviceProfile[];
}

/** 工作区中保存的单个窗口快照 */
export interface WindowSnapshot {
  label: string;
  incognito: boolean;
  url: string;
  deviceId: string;
  orientation: Orientation;
  history: string[];
  historyIndex: number;
}

/** 工作区：一台设备上的整套窗口布局 */
export interface Workspace {
  id: string;
  name: string;
  savedAt: number;
  windows: WindowSnapshot[];
}

export interface DeviceView {
  id: string;
  label: string;
  incognito: boolean;
  url: string;
  deviceId: string;
  orientation: Orientation;
  /** 递增此值可强制重建 iframe，实现彻底重载 */
  reloadToken: number;
  /** 该窗口独立的历史栈，避免使用跨源 iframe history 时的安全限制 */
  history: string[];
  historyIndex: number;
}
