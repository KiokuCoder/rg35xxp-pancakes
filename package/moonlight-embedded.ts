import {appendFile} from "node:fs/promises";
import {$} from "bun"
import {cmakeArgs, cmakeBuild, mesonBuild, mesonCrossFile, v} from "../scripts/helper.ts";
import type {Pack} from "../scripts/pkg.ts";
import {join} from "path";


export const moonlightEmbedded: Pack = {
    name: "moonlight-embedded",
    version: "2.7.0",
    description: "",
    enable: false,
    dependencies: [
        v('opus'),
        v('curl'),
        v('avahi'),
        v('expat'),
        v('openssl'),
        v('alsa'),
        v('libevdev'),
        // v('sdl2'),
    ],
    async sync({download}): Promise<void> {
    },
    async make({resource, rootfs, tar, build}): Promise<void> {
        const src = build("moonlight-embedded/__build")
        await $`[ ! -e ${src} ] || rm -r ${src}`;
        const args: string[] = [];
        const env = {
            ...process.env,
            PKG_CONFIG_PATH: "",
            PKG_CONFIG_LIBDIR: `${rootfs()}/usr/lib/pkgconfig:${rootfs()}/usr/share/pkgconfig`,
            PKG_CONFIG_SYSROOT_DIR: rootfs()
        }
        await $`mkdir -p ${src}`.cwd(build());
        await $`cmake -G"Unix Makefiles" ${cmakeArgs(rootfs())} ..`.env(env).cwd(src)
        await $`make -j16`.cwd(src);
        await $`make install DESTDIR=${rootfs()}`.cwd(src)

    }
}