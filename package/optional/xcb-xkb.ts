import {appendFile} from "node:fs/promises";
import {$} from "bun"
import {cmakeArgs, cmakeBuild, configureMakeMakeInstall, mesonBuild, mesonCrossFile, v} from "../../scripts/helper.ts";
import type {Pack} from "../../scripts/pkg.ts";
import {join} from "path";


const version = 'libxcb-1.16';
export const libxcb: Pack = {
    name: "libxcb",
    version: "1.16",
    description: "XCB utility library - XKB extension",
    enable: true,
    dependencies: [
        v("libXau")
    ],
    async sync({download}): Promise<void> {
        await download({
            url: `https://xcb.freedesktop.org/dist/xcb-proto-1.17.0.tar.gz`,
            filename: `xcb-proto-1.17.0.tar.gz`
        })
        return download({
            url: `https://xcb.freedesktop.org/dist/${version}.tar.gz`,
            filename: `${version}.tar.gz`
        })
    },
    async make({rootfs, tar, build, env_autoconf}): Promise<void> {
        await $`[ ! -e xcb-proto-1.17.0 ] || rm -r xcb-proto-1.17.0`.cwd(build());
        await $`tar -xf ${tar('xcb-proto-1.17.0.tar.gz')}`.cwd(build());
        await configureMakeMakeInstall(build('xcb-proto-1.17.0'), rootfs(), env_autoconf(), [])

        await $`[ ! -e ${build(version)} ] || rm -r ${build(version)}`.cwd(build());
        await $`tar -xf ${tar(`${version}.tar.gz`)}`.cwd(build());
        await configureMakeMakeInstall(build(version), rootfs(), env_autoconf(), [])
    }
}