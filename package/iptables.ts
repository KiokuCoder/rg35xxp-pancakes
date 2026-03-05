import {$} from "bun"
import type {Pack} from "../scripts/pkg.ts";
import {configureMakeMakeInstall} from "../scripts/helper.ts";


const version = 'iptables-1.8.9';
export const iptables: Pack = {
    name: "iptables",
    version: "1.8.9",
    description: "iptables is a user-space utility program that allows systems administrators to configure the tables provided by the Linux kernel firewall (implemented as Netfilter) and the chains and rules it stores. Different kernel modules and programs are currently used for different protocols.",
    enable: false,
    dependencies: [
        {name: "libnftnl", version: "*", required: true}
    ],
    async sync({download}): Promise<void> {
        return download({
            url: `https://www.netfilter.org/projects/iptables/files/${version}.tar.xz`,
            filename: `${version}.tar.xz`
        })
    },
    async make({rootfs, tar, build, env_autoconf}): Promise<void> {
        await $`[ ! -e ${build(version)} ] || rm -r ${build(version)}`.cwd(build());
        await $`tar -xf ${tar(`${version}.tar.xz`)}`.cwd(build());
        await configureMakeMakeInstall(build(version), rootfs(), env_autoconf({}), [])
    }
}
