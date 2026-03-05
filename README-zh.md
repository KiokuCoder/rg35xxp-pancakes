# RG35XX Plus 自定义固件编译系统

[English](README.md) | [简体中文](README-zh.md)

本项目是一个针对 [Anbernic RG35XX Plus](https://linux-sunxi.org/Anbernic_RG35XX_Plus) 的自定义固件编译系统，使用 TypeScript 编写。旨在提供一种更现代、更具可编程性的构建方案，通过高级脚本逻辑管理固件开发流程。

## 项目目标

1.  **独立的编译环境：** 使用 TypeScript (配合 [Bun](https://bun.sh/)) 编写编译逻辑，告别复杂的 Shell 脚本。
2.  **GPU 加速：** 在 RG35XX Plus 上实现了 [Iced](https://iced.rs/) (跨平台 GUI 库) 运行，并带有完整的 GPU 硬件加速。
3.  **核心功能：** 实现了启动、运行游戏、休眠及关机等基本功能。
4.  **开发阶段：** 请注意，本项目目前仍充满各种问题，**不适合日常使用**，仅供开发和研究参考。

## 技术栈

-   **语言：** TypeScript
-   **运行时：** Bun
-   **编译容器：** Docker / Podman (基于 Debian)
-   **目标架构：** aarch64-linux-gnu

## 目录说明

-   `output/`: 编译过程产生的中间文件（请勿直接读取）。
-   `package/`: 软件包描述文件存放路径（如 `zlib.ts`, `sdl2.ts`）。
    -   `package/index.ts`: 软件包注册中心，负责依赖管理。
    -   `package/all.ts`: 虚拟包，定义了默认编译的组件列表。
-   `resource/`: 内置静态资源、配置文件或二进制文件。
    -   `resource/apps/`: 存放自定义应用程序的源代码（主要是 Rust 项目）。
-   `scripts/`: 框架核心编译脚本。
    -   `scripts/toolchain.ts`: 核心工具链 (GCC/G++) 制作脚本。
-   `pack-one.ts`: 用于在开发过程中测试和打包单个软件包。

## 编译流程

### 1. 构建 Docker 镜像
首先基于 Debian 构建包含必备组件的镜像。
```bash
podman build -t rg35xxp-builder .
```

### 2. 启动编译实例
启动容器并挂载当前项目目录。
```bash
podman run --rm -v .:/work -it rg35xxp-builder:latest
```

### 3. 构建工具链
在镜像内部运行以下命令来构建 `aarch64-linux-gnu` 工具链。成功后会生成 `output` 目录，包含 GCC 和 glibc。
```bash
bun toolchain.ts
```

### 4. 编译整个系统
编译 `package/all.ts` 中定义的所有组件。
```bash
bun pack-one.ts all
```
如果一切顺利，`output/rootfs` 目录下将生成包含最低限度组件的根文件系统。

## 刷机与部署

部署固件需要使用官方镜像作为基础：
1.  下载 [RG35XX+-P-V1.2.3-EN16GB-251225.IMG.7z](https://drive.google.com/drive/folders/1LUFdm1ZXKbWIVGd2G4Qd5Pa6CA-peUdx)。
2.  解压得到 `.IMG` 文件。
3.  将 `output/rootfs` 下的所有文件替换到镜像的第 5 分区 (p5, ext4 格式, 约 7GB)。

你可以使用以下命令直接更新镜像文件：
```bash
mke2fs -t ext4 -d output/rootfs -E offset=2302672896 RG35XX+-P-V1.2.3-EN16GB-251225.IMG 7G
```

## 开发建议

软件包通过 `Pack` 接口进行描述。您可以参考 `package/zlib.ts` 或 `package/busybox.ts` 来了解如何编写软件的同步（下载）和编译（make/cmake/meson）流程。

---
*免责声明：本项目按“原样”提供，使用风险自担。*
