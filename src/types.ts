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

export interface DeviceView {
  id: string;
  label: string;
  incognito: boolean;
  url: string;
  deviceId: string;
  orientation: Orientation;
  /**
   * 存储隔离模式：iframe 不授予 allow-same-origin，获得独立的不透明源，
   * Cookie / localStorage 与其它窗口完全隔离，但依赖本地存储的页面可能无法正常运行。
   */
  isolated: boolean;
  /** 递增此值可强制重建 iframe，实现彻底重载 */
  reloadToken: number;
  /** 该窗口独立的历史栈，避免使用跨源 iframe history 时的安全限制 */
  history: string[];
  historyIndex: number;
}
