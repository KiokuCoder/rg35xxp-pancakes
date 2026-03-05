import {appendFile} from "node:fs/promises";
import {$} from "bun"
import {cmakeArgs, cmakeBuild, configureMakeMakeInstall, mesonBuild, mesonCrossFile, v} from "../../scripts/helper.ts";
import type {Pack} from "../../scripts/pkg.ts";
import {join} from "path";


const version = 'xproto-7.0.31';
export const xproto: Pack = {
    name: "xproto",
    version: "7.0.31",
    description: "X Window System core protocol",
    enable: true,
    dependencies: [],
    async sync({download}): Promise<void> {
        return download({
            url: `https://xorg.freedesktop.org/releases/individual/proto/${version}.tar.bz2`,
            filename: `${version}.tar.bz2`
        })
    },
    async make({rootfs, tar, build, env_autoconf}): Promise<void> {
        await $`[ ! -e ${build(version)} ] || rm -r ${build(version)}`.cwd(build());
        await $`tar -xf ${tar(`${version}.tar.bz2`)}`.cwd(build());
        await $`rm config.sub`.env(env_autoconf()).cwd(build(version));
        await $`autoreconf -fi`.env(env_autoconf()).cwd(build(version));
        await configureMakeMakeInstall(build(version), rootfs(), env_autoconf(), [])
    }
}