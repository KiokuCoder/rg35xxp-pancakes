import {$} from "bun"
import type {Pack} from "../scripts/pkg.ts";
import {configureMakeMakeInstall} from "../scripts/helper.ts";


const version = 'libnftnl-1.2.1';
export const libnftnl: Pack = {
    name: "libnftnl",
    version: "1.2.1",
    description: "libnftnl is a userspace library providing an API to the nf_tables subsystem in the Linux kernel.",
    enable: false,
    dependencies: [],
    async sync({download}): Promise<void> {
        return download({
            url: `https://www.netfilter.org/projects/libnftnl/files/${version}.tar.bz2`,
            filename: `${version}.tar.bz2`
        })
    },
    async make({rootfs, tar, build, env_autoconf}): Promise<void> {
        await $`[ ! -e ${build(version)} ] || rm -r ${build(version)}`.cwd(build());
        await $`tar -xf ${tar(`${version}.tar.bz2`)}`.cwd(build());
        await configureMakeMakeInstall(build(version), rootfs(), env_autoconf(), [])
    }
}
