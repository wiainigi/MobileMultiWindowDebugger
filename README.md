# MobileMultiWindowDebugger

移动端多窗口调试器 —— 基于 Tauri 2 + Vue 3 的桌面工具，在同一个界面内并排开启多个手机尺寸的浏览窗口，用于移动端页面适配、多会话状态对比与布局验证。

## 功能特性

**多窗口舞台**
- 同一界面内横向排列多个浏览器窗口，每个窗口可独立设置机型分辨率、横竖屏、URL，并各自持有独立的浏览历史
- 内置 22 种常用移动端分辨率预设，按 `iOS` / `Android` / `HarmonyOS · 国产` / `小程序容器` 分组
- 「适应宽度」可按舞台可用宽高自动等比缩放整排窗口；关闭时按 100% 显示并横向滚动
- 支持单行排列与自动换行两种布局

**无痕窗口与存储隔离**
- 每个无痕窗口经**独立端口的本地代理**加载，页面 origin 变为 `http://127.0.0.1:<独立端口>`
- 因此各无痕窗口之间、无痕与普通窗口之间的 Cookie / localStorage / IndexedDB **完全隔离**，关闭窗口即销毁
- 无痕窗口以黑紫主题与「无痕」标签区分，普通窗口为黑蓝主题，活动窗口使用更亮一档的底色

**导航与信息**
- 每个窗口顶栏分两行：网站标题（超出省略）+ 无痕标签；机型选择、宽×高、旋转、刷新、开发者工具、系统浏览器打开、关闭
- 地址栏作用于当前活动窗口，支持勾选「同步全部」广播到所有窗口
- 网站标题由 Rust 侧代理请求解析（同源页面直接读取运行时标题），跨源站点也能正确显示

**工作区**
- 保存 / 更新 / 切换 / 删除工作区，完整记录窗口数量、普通与无痕类型、URL、机型、朝向与各自历史
- 载入或新建工作区前若有已打开窗口，会提示"未保存的改动会丢失"并要求确认

**其他**
- 每个窗口提供 `>_` 按钮打开开发者工具
- 内置默认页 `default.html`，所有新窗口均从该页开始，离线可用
- 顶部菜单栏：文件 / 视图 / 窗口 / 帮助，含快捷键

## 技术栈

| 层 | 技术 |
|---|---|
| 桌面框架 | Tauri 2（`devtools` feature 启用） |
| 前端 | Vue 3 `<script setup>` + TypeScript + Vite，纯 CSS 深色主题，无 UI 库 |
| 后端 | Rust（`hyper` 1 本地代理、`reqwest` 上游请求、`tokio` 异步运行时） |
| 图标与打包 | WiX 3.14（MSI）、NSIS（安装程序） |

## 目录结构

```
├── index.html                    主界面入口
├── public/
│   ├── default.html              内置默认页（所有新窗口的起始页）
│   └── icon.ico                  应用图标源文件
├── src/
│   ├── App.vue                   整体布局：菜单栏 / 工具栏 / 舞台 / 状态栏
│   ├── types.ts                  数据模型：DeviceProfile、DeviceView、Workspace 等
│   ├── constants/devices.ts      22 种移动端机型分辨率预设
│   ├── composables/
│   │   ├── useDeviceManager.ts   窗口状态中枢：增删、导航、历史、快照与恢复
│   │   └── useWorkspaces.ts      工作区管理（localStorage 持久化）
│   ├── components/
│   │   ├── MenuBar.vue           顶部菜单栏 + 工作区按钮组与确认弹窗
│   │   ├── ToolBar.vue           地址栏与窗口管理工具
│   │   ├── DeviceStage.vue       多窗口舞台，计算自适应缩放
│   │   ├── DeviceFrame.vue       单个窗口：标题栏、iframe、状态栏
│   │   └── AboutDialog.vue       关于对话框
│   └── styles/main.css           全局样式与主题变量
└── src-tauri/
    ├── src/
    │   ├── main.rs               入口
    │   ├── lib.rs                命令注册：站点标题、代理生命周期、开发者工具
    │   └── proxy.rs              每窗口独立的本地反向代理
    ├── tests/proxy_smoke.rs      代理行为集成测试
    ├── capabilities/default.json 权限配置
    ├── icons/                    应用图标（打包使用）
    └── tauri.conf.json           窗口尺寸、打包目标等配置
```

## 环境要求

- Node.js 20+ 与 npm
- Rust 稳定版工具链（`cargo`、`rustc`）
- Windows：WebView2 Runtime（Windows 11 自带，Windows 10 需单独安装）
- 打包额外依赖 WiX 3.14 与 NSIS，Tauri CLI 会自动下载并缓存到 `%LOCALAPPDATA%\tauri`（首次打包需要联网）

## 快速开始

```powershell
# 安装前端依赖
npm install

# 开发调试（自动拉起 Vite 并启动原生窗口，前后端均支持热更新）
npm run tauri dev

# 仅运行前端（浏览器中预览界面，Tauri 相关能力不可用）
npm run dev

# 前端类型检查与构建
npm run build
```

## 构建 Windows 安装包

```powershell
npm run tauri -- build
```

产物位于 `src-tauri/target/release/`：

| 文件 | 说明 |
|---|---|
| `mobile-multi-window-debugger.exe` | 免安装可执行文件 |
| `bundle/msi/MobileMultiWindowDebugger_0.1.0_x64_en-US.msi` | MSI 安装包 |
| `bundle/nsis/MobileMultiWindowDebugger_0.1.0_x64-setup.exe` | NSIS 安装程序 |

```powershell
# 只构建某一种安装包（可选 msi / nsis）
npm run tauri -- build --bundles nsis

# 只编译可执行文件，不生成安装包
npm run tauri -- build --no-bundle
```

## 使用说明

### 快捷键

| 快捷键 | 功能 |
|---|---|
| `Ctrl + Shift + N` | 新建无痕窗口 |
| `Ctrl + N` | 新建普通窗口 |
| `Ctrl + W` | 关闭当前窗口 |
| `Ctrl + R` | 刷新当前窗口 |

### 窗口操作

- 点击任意窗口（含 iframe 内容区域）即将其设为活动窗口，地址栏、状态栏随之同步
- 地址栏回车只作用于活动窗口；勾选「同步全部」可把地址广播到所有窗口
- 窗口顶栏第二行左侧选择机型、右侧依次为：旋转屏幕、刷新、开发者工具、在默认浏览器中打开、关闭

### 无痕窗口的隔离原理

浏览器中 `Cookie` / `localStorage` 是按 **origin** 隔离的，而 iframe 直接加载目标站点无法改变 origin。因此本项目为每个无痕窗口在 `127.0.0.1` 上分配一个独立端口作为反向代理，使每个无痕窗口拥有独一无二的 origin：

```
无痕窗口 1  →  http://127.0.0.1:51234/  →  代理  →  http://localhost:5173/
无痕窗口 2  →  http://127.0.0.1:51235/  →  代理  →  http://localhost:5173/
```

代理同时负责：改写 `Set-Cookie`（去掉 `Domain`/`Secure` 让 Cookie 归属代理 origin）、改写 `Location` 跳转、移除 `X-Frame-Options` 与 `Content-Security-Policy`（使原本禁止嵌入的站点也能显示）、以及 WebSocket 隧道（保证 Vite HMR 可用）。

### 工作区

- 「工作区 ▾」下拉选择已保存的工作区，每项右侧 `✕` 可删除
- 「保存工作区」在未打开工作区时弹出命名框；已打开时按钮变为「更新工作区」，点击直接覆盖保存
- 「新建工作区」清空当前窗口布局并解除与已保存工作区的关联
- 载入 / 新建前若有已打开窗口，会弹出确认提示，避免丢失未保存的布局

数据存放于 WebView 的 `localStorage`（键 `mmwd.workspaces.v1`），重启应用后保留；开发态与打包态的 origin 不同，两边的数据不互通。

### 开发者工具

点击窗口顶栏的 `>_` 按钮打开开发者工具。受浏览器限制，iframe 无法拥有独立的 DevTools 窗口，因此打开的是承载全部调试窗口的 WebView 的 DevTools；请在其 **Console 左上角的上下文选择器** 中切换到对应窗口（每个 iframe 已按窗口名设置了 `name`，便于识别）。release 构建同样可用（依赖 `tauri` 的 `devtools` feature）。

## 已知限制

- **站点嵌入限制**：普通窗口直接使用 iframe，目标站点若设置了 `X-Frame-Options` 或 CSP `frame-ancestors` 将无法显示，可改用无痕窗口（经代理可绕过）或点「↗」在系统浏览器打开
- **地址栏不会跟随页内跳转**：跨源 iframe 内部 URL 无法读取，窗口内的链接跳转、SPA 路由变化不会回写到地址栏；前进/后退基于本工具维护的历史栈
- **https 站点的 WebSocket 未支持**：代理的 WebSocket 隧道目前仅支持 `http` 上游（本地开发场景），`https` 站点的 HMR 会在无痕窗口内失效
- **默认页与主界面同源**：默认页本身不做存储隔离，其 `localStorage` 与宿主共享
- **安装包**：未做代码签名，首次运行会有 SmartScreen 提示；MSI 默认语言为英文，可在 `tauri.conf.json` 中通过 `bundle.windows.wix.language` 改为 `zh-CN`
- **图标**：当前 `icon.ico` 最大尺寸为 64×64，如需在大图标视图更清晰，可执行 `npm run tauri -- icon <1024×1024 的 PNG>` 重新生成整套图标

## 开发提示

- Rust 侧 `proxy.rs` 的行为由集成测试覆盖，可执行 `cd src-tauri && cargo test --test proxy_smoke` 验证转发、Cookie 改写、重定向改写与端口释放
- 前端若将 Rust 命令改名或增删，需同步更新 `src-tauri/src/lib.rs` 的 `invoke_handler` 注册
- 使用 VS Code 时推荐安装 [Vue - Official](https://marketplace.visualstudio.com/items?itemName=Vue.volar)、[Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) 与 [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)；请勿启用已过时的 Vetur 扩展（会对 Vue 3 语法产生误报）
