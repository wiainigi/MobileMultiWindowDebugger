import type { DeviceGroup, DeviceProfile } from "../types";

/**
 * 常用移动端 CSS 逻辑分辨率（单位 px，非物理像素）。
 */
const RAW_DEVICES: DeviceProfile[] = [
  // ---------- iOS ----------
  { id: "iphone-5", name: "iPhone 5 / SE1", group: "iOS", width: 320, height: 568, dpr: 2 },
  { id: "iphone-se", name: "iPhone SE (2020/2022)", group: "iOS", width: 375, height: 667, dpr: 2 },
  { id: "iphone-13-mini", name: "iPhone 13 mini", group: "iOS", width: 375, height: 812, dpr: 3 },
  { id: "iphone-14", name: "iPhone 13 / 14", group: "iOS", width: 390, height: 844, dpr: 3 },
  { id: "iphone-14-pro", name: "iPhone 14 Pro", group: "iOS", width: 393, height: 852, dpr: 3 },
  { id: "iphone-xr", name: "iPhone XR / 11", group: "iOS", width: 414, height: 896, dpr: 2 },
  { id: "iphone-14-pro-max", name: "iPhone 14 Pro Max", group: "iOS", width: 430, height: 932, dpr: 3 },
  { id: "ipad-mini", name: "iPad mini", group: "iOS", width: 768, height: 1024, dpr: 2 },
  { id: "ipad-air", name: "iPad Air", group: "iOS", width: 820, height: 1180, dpr: 2 },
  { id: "ipad-pro-129", name: "iPad Pro 12.9\"", group: "iOS", width: 1024, height: 1366, dpr: 2 },

  // ---------- Android ----------
  { id: "android-360x640", name: "Android 通用 360×640", group: "Android", width: 360, height: 640, dpr: 2 },
  { id: "galaxy-s8", name: "Galaxy S8 / S9", group: "Android", width: 360, height: 740, dpr: 3 },
  { id: "galaxy-s20", name: "Galaxy S20 / S21", group: "Android", width: 360, height: 800, dpr: 3 },
  { id: "galaxy-s22-ultra", name: "Galaxy S22 Ultra", group: "Android", width: 384, height: 854, dpr: 3.5 },
  { id: "pixel-5", name: "Pixel 5", group: "Android", width: 393, height: 851, dpr: 2.75 },
  { id: "pixel-7", name: "Pixel 7 / 7 Pro", group: "Android", width: 412, height: 915, dpr: 2.6 },
  { id: "galaxy-tab-s8", name: "Galaxy Tab S8", group: "Android", width: 800, height: 1280, dpr: 2 },

  // ---------- HarmonyOS / 国产机型 ----------
  { id: "mate-60-pro", name: "HUAWEI Mate 60 Pro", group: "HarmonyOS / 国产", width: 393, height: 873, dpr: 3 },
  { id: "xiaomi-14", name: "Xiaomi 14", group: "HarmonyOS / 国产", width: 393, height: 873, dpr: 3 },
  { id: "vivo-x100", name: "vivo X100", group: "HarmonyOS / 国产", width: 412, height: 915, dpr: 3 },
  { id: "huawei-matepad", name: "HUAWEI MatePad 11", group: "HarmonyOS / 国产", width: 800, height: 1232, dpr: 2 },

  // ---------- 小程序 / 轻量容器 ----------
  { id: "miniprogram", name: "微信小程序 375×667", group: "小程序容器", width: 375, height: 667, dpr: 2 },
  { id: "uniapp", name: "uni-app 移动 web", group: "小程序容器", width: 375, height: 812, dpr: 2 },
];

export const DEFAULT_DEVICE_ID = "iphone-14";

export const DEVICE_PRESETS: DeviceProfile[] = RAW_DEVICES;

export const DEVICE_GROUPS: DeviceGroup[] = RAW_DEVICES.reduce<DeviceGroup[]>((groups, device) => {
  const existing = groups.find((group) => group.name === device.group);
  if (existing) {
    existing.devices.push(device);
  } else {
    groups.push({ name: device.group, devices: [device] });
  }
  return groups;
}, []);

export function findDevice(deviceId: string): DeviceProfile {
  return DEVICE_PRESETS.find((device) => device.id === deviceId) ?? DEVICE_PRESETS[0];
}
