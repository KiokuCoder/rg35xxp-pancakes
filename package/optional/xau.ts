import {appendFile} from "node:fs/promises";
import {$} from "bun"
import {cmakeArgs, cmakeBuild, configureMakeMakeInstall, mesonBuild, mesonCrossFile, v} from "../../scripts/helper.ts";
import type {Pack} from "../../scripts/pkg.ts";
import {join} from "path";


const version = 'libXau-1.0.11';
export const xau: Pack = {
    name: "libXau",
    version: "1.0.9",
    description: "libXau - X11 Authorization Protocol library",
    enable: true,
    dependencies: [v("xproto")],
    async sync({download}): Promise<void> {
        return download({
            url: `https://www.x.org/releases/individual/lib/${version}.tar.gz`,
            filename: `${version}.tar.gz`
        })
    },
    async make({rootfs, tar, build, env_autoconf}): Promise<void> {
        await $`[ ! -e ${build(version)} ] || rm -r ${build(version)}`.cwd(build());
        await $`tar -xf ${tar(`${version}.tar.gz`)}`.cwd(build());
        await configureMakeMakeInstall(build(version), rootfs(), env_autoconf(), [])
    }
}