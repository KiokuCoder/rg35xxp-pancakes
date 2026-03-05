import {$} from "bun"
import {mesonBuild, v} from "../../scripts/helper.ts";
import type {Pack} from "../../scripts/pkg.ts";

const version = 'xkbcommon-1.7.0';
export const xkbcommon: Pack = {
    name: "xkbcommon",
    version: "1.7.0",
    description: "keymap handling library for toolkits and window systems",
    enable: false,
    dependencies: [],
    async sync({download}): Promise<void> {
        return download({
            url: `https://github.com/xkbcommon/libxkbcommon/archive/refs/tags/${version}.tar.gz`,
            filename: `${version}.tar.gz`
        })
    },
    async make({resource, rootfs, tar, build, env_autoconf}): Promise<void> {
        const src = build('libxkbcommon-' + version);
        await $`[ ! -e ${src} ] || rm -r ${src}`.cwd(build());
        await $`tar -xf ${tar(`${version}.tar.gz`)}`.cwd(build());
        await mesonBuild(src, rootfs(), ['-Denable-x11=false', '-Denable-wayland=true', '-Denable-docs=false'])
    }
}
