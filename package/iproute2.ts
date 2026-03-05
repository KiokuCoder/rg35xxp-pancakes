import {$} from "bun"
import {configureMakeMakeInstall} from "../scripts/helper.ts";
import type {Pack} from "../scripts/pkg.ts";

const version = 'iproute2-6.7.0';
export const iproute2: Pack = {
    name: "iproute2",
    version: "6.7.0",
    description: "Iproute2 is a collection of utilities for controlling TCP/IP networking and traffic control in Linux.",
    enable: true,
    dependencies: [],
    async sync({download}): Promise<void> {
        return download({
            url: `https://mirrors.edge.kernel.org/pub/linux/utils/net/iproute2/${version}.tar.xz`,
            filename: `${version}.tar.xz`
        })
    },
    async make({resource, rootfs, tar, build, env_autoconf}): Promise<void> {
        await $`[ ! -e ${build(version)} ] || rm -r ${build(version)}`.cwd(build());
        await $`tar -xf ${tar(`${version}.tar.xz`)}`.cwd(build());
        await $`./configure --prefix=/usr`.env(env_autoconf()).cwd(build(version));
        await $`make -j16`.cwd(build(version));
        await $`make install DESTDIR=${rootfs()}`.cwd(build(version));
    }
}
