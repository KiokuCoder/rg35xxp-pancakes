import {$} from "bun"
import {mesonBuild, v} from "../scripts/helper.ts";
import type {Pack} from "../scripts/pkg.ts";

const version = 'libinput-1.19.4';
export const libinput: Pack = {
    name: "libinput",
    version: "1.19.4",
    description: "libinput is a library to handle input devices",
    enable: false,
    dependencies: [v("eudev"), v("mtdev"), v("libevdev")],
    async sync({download}): Promise<void> {
        return download({
            url: `https://www.freedesktop.org/software/libinput/${version}.tar.xz`,
            filename: `${version}.tar.xz`
        })
    },
    async make({resource, rootfs, tar, build, env_autoconf}): Promise<void> {
        await $`[ ! -e ${build(version)} ] || rm -r ${build(version)}`.cwd(build());
        await $`tar -xf ${tar(`${version}.tar.xz`)}`.cwd(build());
        const args: string[] = [
            '-Ddocumentation=false',
            '-Dtests=false',
            '-Ddebug-gui=false',
            '-Dlibwacom=false'
        ]
        await mesonBuild(build(version), rootfs(), args);
    }
}