import {$} from "bun"
import {cmakeBuild} from "../scripts/helper.ts";
import type {Pack} from "../scripts/pkg.ts";

const version = 'libADLMIDI-1.5.1';
export const libADLMIDI: Pack = {
    name: "libadlmidi",
    version: "1.5.1",
    description: "libADLMIDI is a MIDI player library with OPL3 (YMF262) emulation, used by vmrp to play mrp background music.",
    enable: false,
    dependencies: [],
    async sync({download}): Promise<void> {
        return download({
            url: `https://github.com/Wohlstand/libADLMIDI/archive/refs/tags/v${version.split('-')[1]}.tar.gz`,
            filename: `${version}.tar.gz`
        })
    },
    async make({rootfs, tar, build}): Promise<void> {
        await $`[ ! -e ${build(version)} ] || rm -r ${build(version)}`.cwd(build());
        await $`tar -xf ${tar(`${version}.tar.gz`)}`.cwd(build());
        await cmakeBuild(build(version), rootfs(), {}, [
            // 上游默认只出静态库，而且 GNUInstallDirs 在交叉编译下会把库装到
            // usr/lib/aarch64-linux-gnu，和 rootfs 里其它包对不上
            "-DCMAKE_INSTALL_LIBDIR=lib",
            "-DlibADLMIDI_SHARED=ON",
            "-DlibADLMIDI_STATIC=OFF",
            // 只保留 DosBox 这个轻量的 OPL3 实现，掌机上跑 Nuked 太吃 CPU，
            // 用哪个由 sound.c 的 adl_switchEmulator() 指定
            "-DUSE_NUKED_EMULATOR=OFF",
            "-DUSE_OPAL_EMULATOR=OFF",
            "-DUSE_JAVA_EMULATOR=OFF",
        ])
    }
}
