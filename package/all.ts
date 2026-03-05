import type {Pack} from "../scripts/pkg.ts";
import {v} from "../scripts/helper.ts";
import uutils from "./uutils.ts";

export const all: Pack = {
    name: "all",
    version: "1.0.0",
    description: "",
    enable: false,
    dependencies: [
        // 系统组件
        v("skeleton"),
        v("rg35xxp-firmware"),
        v("uutils-coreutils"),
        v("uutils-findutils"),
        v("util-linux"),
        v("kmod"),
        v("procps"),
        v("iproute2"),
        v("wpa_supplicant"),
        v("alsa"),
        v("alsa-utils"),
        v("bash"),
        v("adb"),
        v("openssh"),
        v("dropbear"),
        v("busybox"), // 我费了很大劲，要去除这个玩意，但是始终哪里缺少一点

        // 自带组件
        v("rg35xxp-init"),
        v("rg35xxp-launcher"),
        v("rg35xxp-guard"),

        // 扩展组件
        v("mpv"),
        v("ffmpeg"),
        v("tmux"),
        v("RetroArch"),
        v("RetroArchAssets"),
        v("ppsspp"),
        v("mgba"),
        v("love2d"),
    ],
    async sync(): Promise<void> {
    },
    async make(): Promise<void> {
    }
}
