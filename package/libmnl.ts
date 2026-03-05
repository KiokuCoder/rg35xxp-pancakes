import {$} from "bun"
import {configureMakeMakeInstall} from "../scripts/helper.ts";
import type {Pack} from "../scripts/pkg.ts";

const version = 'libmnl-1.0.4';
export const libmnl: Pack = {
    name: "libmnl",
    version: "1.0.4",
    description: "libmnl is a minimalistic netlink library.",
    enable: false,
    dependencies: [],
    async sync({download}): Promise<void> {
        return download({
            url: `https://www.netfilter.org/projects/libmnl/files/${version}.tar.bz2`,
            filename: `${version}.tar.bz2`
        })
    },
    async make({resource, rootfs, tar, build, env_autoconf}): Promise<void> {
        await $`[ ! -e ${build(version)} ] || rm -r ${build(version)}`.cwd(build());
        await $`tar -xf ${tar(`${version}.tar.bz2`)}`.cwd(build());

        await configureMakeMakeInstall(build(version), rootfs(), env_autoconf(), [])
    }
}
