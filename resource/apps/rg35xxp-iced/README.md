# RG35XXP-Iced Launcher

这是一个为 **Anbernic RG35XX Plus** 掌机量身定制的高性能游戏启动器（Launcher）。

### 进度

目前实现了最低限度的功能

1. 游戏启动
2. 软件启动
3. 常用设定

但是需要实现的功能比我预想得要多得多...


### 已实现页面

1. GBA 游戏页面
2. 软件启动页面
3. 设定页面（音量、亮度、WiFi）
4. 文件管理

### 未来计划

- [ ] 游戏启动页面（包含 GBA/PSP 等不同类目游戏浏览；收藏夹；历史记录）
- [ ] 软件启动页面
- [ ] 系统设定
  - 时区
  - 语言
  - 亮度
  - 音量
  - 屏幕超时
  - 修改壁纸
  - 主题（待定？）
  - SSH 启用禁用、密码设定
  - 服务启用禁用
  - 无线网络链接管理
  - 系统信息查看（IP、存储、运行内存等）
  - 关机、重启

- [ ] UI 组件: 对话框、确认框、菜单组件、列表组件、长内容滚动组件
- [ ] 文件管理器（移动、复制、重命名、文件信息、图片预览、外部程序打开视频、音频）


### 设计思路

我使用 [wgpu-hal](https://crates.io/crates/wgpu-hal) 抽象层配合 [egl](src/rg35xxp/egl.rs) 驱动，实现了一个 iced-renderer，并且借助
[UserInterface](https://docs.iced.rs/iced_runtime/user_interface/struct.UserInterface.html) 手动实现事件循环。
在无按键输入的情况下进程保持低于 `2%` 的 CPU 占用，在按键触发事件需要重新渲染的时候，CPU 占用会保持在 `6%` 左右之后回落。