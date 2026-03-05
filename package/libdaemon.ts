import {appendFile, exists} from "node:fs/promises";
import {$} from "bun"
import {cmakeArgs, cmakeBuild, configureMakeMakeInstall, mesonBuild, mesonCrossFile, v} from "../scripts/helper.ts";
import type {Pack} from "../scripts/pkg.ts";
import {join} from "path";


const version = 'libdaemon-0.14';
export const libdaemon: Pack = {
    name: "libdaemon",
    version: "0.14",
    description: "A lightweight C library that eases the writing of UNIX daemons",
    enable: true,
    dependencies: [],
    async sync({download, build}): Promise<void> {
        if (await exists(build('libdaemon-0.14'))) return
        await $`git clone --depth 1 https://github.com/thingnario/libdaemon-0.14.git`.cwd(build())
    },
    async make({rootfs, tar, build, env_autoconf}): Promise<void> {
        const args0 = [

            // 安装目录
            '--prefix=/usr',
            '--host=aarch64-linux-gnu',
            '--disable-static',
            '--enable-shared',
        ]
        await $`rm config.sub`.env(env_autoconf()).cwd(build(version));
        await $`autoreconf -fi`.env(env_autoconf()).cwd(build(version));
        await $`./configure ${args0}`.env(env_autoconf()).cwd(build(version));
        await $`make -j16`.env(env_autoconf()).cwd(build(version));
        await $`make install DESTDIR=${rootfs()}`.env(env_autoconf()).cwd(build(version));
    }
}