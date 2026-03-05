import {$} from "bun"
import {cmakeBuild, v} from "../scripts/helper.ts";
import type {Pack} from "../scripts/pkg.ts";

const version = 'ppsspp-1.19.3';
export const ppsspp: Pack = {
    name: "ppsspp",
    version: "1.19.3",
    description: "A fast and portable PSP emulator.",
    enable: false,
    dependencies: [
        v("sdl2"),
        v("ffmpeg"),
        v("zlib"),
    ],
    async sync({download}): Promise<void> {
        return download({
            url: `https://github.com/hrydgard/ppsspp/releases/download/v${version.split('-')[1]}/${version}.tar.xz`,
            filename: `${version}.tar.xz`
        })
    },
    async make({rootfs, tar, build, resource}): Promise<void> {
        await $`[ ! -e ${build(version)} ] || rm -r ${build(version)}`.cwd(build());
        await $`tar -xf ${tar(`${version}.tar.xz`)}`.cwd(build());

        await cmakeBuild(build(version), rootfs(), {}, [
            "-DUSING_FBDEV=ON",
            "-DUSING_EGL=ON",
            "-DUSING_GLES2=ON",
            "-DUSE_WAYLAND_WSI=OFF",
            "-DUSING_X11_VULKAN=OFF",
            "-DUSE_DISCORD=OFF",
            "-DUSE_MINIUPNPC=OFF",
            "-DUSING_QT_UI=OFF",
            "-DHEADLESS=OFF",
            "-DUNITTEST=OFF",
            "-DSIMULATOR=OFF",
            "-DUSE_SYSTEM_FFMPEG=ON",
            "-DUSE_SYSTEM_ZLIB=ON",
            "-DUSE_SYSTEM_LIBSDL2=ON",
            "-DCMAKE_BUILD_TYPE=Release",
            `-DEGL_LIBRARY=${rootfs()}/usr/lib64/libmali.so`,
            `-DCMAKE_EXE_LINKER_FLAGS=-L${rootfs()}/usr/lib64 -lEGL -lmali`,
        ])
        await $`install -Dm 0644 ${resource('controls.ini')} ${rootfs('/root/.config/ppsspp/PSP/SYSTEM/controls.ini')}`
    },
}
