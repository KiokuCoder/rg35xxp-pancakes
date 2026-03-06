import {$} from "bun"
import {mesonBuild} from "../../scripts/helper.ts";
import type {Pack} from "../../scripts/pkg.ts";


const version = 'libdrm-2.4.110';
export const libdrm: Pack = {
    name: "libdrm",
    version: "2.4.110",
    description: "Userspace interface to kernel DRM services",
    enable: false,
    dependencies: [],
    async sync({download}): Promise<void> {
        return download({
            url: `https://dri.freedesktop.org/libdrm/${version}.tar.xz`,
            filename: `${version}.tar.xz`
        })
    },
    async make({resource, rootfs, tar, build, env_autoconf}): Promise<void> {
        await $`[ ! -e ${build(version)} ] || rm -r ${build(version)}`.cwd(build());
        await $`tar -xf ${tar(`${version}.tar.xz`)}`.cwd(build());
        await mesonBuild(build(version), rootfs(), []);
    }
}
