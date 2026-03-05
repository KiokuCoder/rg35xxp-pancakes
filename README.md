# RG35XX Plus Custom Firmware Build System

[English](README.md) | [简体中文](README-zh.md)

This project is a custom firmware build system for the [Anbernic RG35XX Plus](https://linux-sunxi.org/Anbernic_RG35XX_Plus), written in TypeScript. It aims to provide a modern, scriptable approach for firmware development using high-level logic instead of complex shell scripts.

## Project Goals

1.  **Independent Build Environment:** Uses TypeScript (with [Bun](https://bun.sh/)) for build logic instead of complex shell scripts.
2.  **GPU Acceleration:** Implements [Iced](https://iced.rs/) (a cross-platform GUI library) on the RG35XX Plus with full GPU acceleration.
3.  **Core Functionality:** Supports startup, game execution, sleep, and shutdown.
4.  **Proof of Concept:** Please note that this project is still experimental and contains various bugs. It is **not recommended for daily use**.

## Tech Stack

-   **Language:** TypeScript
-   **Runtime:** Bun
-   **Build Container:** Docker / Podman (Debian-based)
-   **Target Architecture:** aarch64-linux-gnu

## Directory Structure

-   `output/`: Intermediate build files (ignore this directory).
-   `package/`: Contains package definition files (e.g., `zlib.ts`, `sdl2.ts`).
    -   `package/index.ts`: Registry for all packages.
    -   `package/all.ts`: A virtual package defining the default components to build.
-   `resource/`: Built-in static resources, configuration files, and binaries.
    -   `resource/apps/`: Source code for custom applications (Rust-based).
-   `scripts/`: Core framework build scripts.
    -   `scripts/toolchain.ts`: Script for building the core GCC/G++ toolchain.
-   `pack-one.ts`: Utility script for building individual packages during development.

## Build Process

### 1. Build the Docker Image
First, build a Debian-based image containing the necessary build tools.
```bash
podman build -t rg35xxp-builder .
```

### 2. Start the Build Container
Run the container and mount the project directory.
```bash
podman run --rm -v .:/work -it rg35xxp-builder:latest
```

### 3. Build the Toolchain
Inside the container, build the `aarch64-linux-gnu` toolchain. This will generate the `output` directory containing GCC and glibc.
```bash
bun toolchain.ts
```

### 4. Build the System
Compile all components defined in `package/all.ts`.
```bash
bun pack-one.ts all
```
If successful, the resulting root filesystem will be available in `output/rootfs`.

## Deployment

To deploy the firmware, you need the official image as a base:
1.  Download [RG35XX+-P-V1.2.3-EN16GB-251225.IMG.7z](https://drive.google.com/drive/folders/1LUFdm1ZXKbWIVGd2G4Qd5Pa6CA-peUdx).
2.  Extract the `.IMG` file.
3.  Replace the contents of the 5th partition (p5, ext4, ~7GB) with the files from `output/rootfs`.

You can use the following command to update the image:
```bash
mke2fs -t ext4 -d output/rootfs -E offset=2302672896 RG35XX+-P-V1.2.3-EN16GB-251225.IMG 7G
```

## Contributing

Packages are defined using a `Pack` interface. See `package/zlib.ts` or `package/busybox.ts` for examples on how to describe software synchronization (downloading) and compilation (make/cmake/meson).

---
*Disclaimer: This project is provided as-is. Use it at your own risk.*
