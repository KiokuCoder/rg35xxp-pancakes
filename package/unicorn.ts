import type {Pack} from "../scripts/pkg.ts";
import {$} from "bun";
import {cmakeBuild} from "../scripts/helper.ts";

const version = 'unicorn-2.1.4';
export const unicorn: Pack = {
    name: "unicorn",
    version: "2.1.4",
    description: "Unicorn is a lightweight, multi-architecture CPU emulator framework based on QEMU.",
    enable: false,
    dependencies: [],
    async sync({download}): Promise<void> {
        return download({
            url: `https://github.com/unicorn-engine/unicorn/archive/refs/tags/${version.split('-')[1]}.tar.gz`,
            filename: `${version}.tar.gz`
        })
    },
    async make({rootfs, tar, build}): Promise<void> {
        await $`[ ! -e ${build(version)} ] || rm -r ${build(version)}`.cwd(build());
        await $`tar -xf ${tar(`${version}.tar.gz`)}`.cwd(build());
        await cmakeBuild(build(version), rootfs(), {}, [
            // 只需要 arm 目标，vmrp 只模拟 arm 代码
            "-DUNICORN_ARCH=arm",
            "-DUNICORN_BUILD_TESTS=OFF",
            "-DUNICORN_INSTALL=ON",
            "-DBUILD_SHARED_LIBS=ON",
            "-DCMAKE_BUILD_TYPE=Release",
        ])
    }
}
