import {$} from "bun"
import {configureMakeMakeInstall} from "../scripts/helper.ts";
import type {Pack} from "../scripts/pkg.ts";

const version = 'strace-6.11';
export const strace: Pack = {
    name: "strace",
    version: "6.11",
    description: "strace is a powerful diagnostic, debugging and instructional userspace utility for Linux",
    enable: false,
    dependencies: [],
    async sync({download}): Promise<void> {
        return download({
            url: `https://github.com/strace/strace/releases/download/v6.11/strace-6.11.tar.xz`,
            filename: `${version}.tar.xz`
        })
    },
    async make({resource, rootfs, tar, build, env_autoconf}): Promise<void> {
        await $`[ ! -e ${build(version)} ] || rm -r ${build(version)}`.cwd(build());
        await $`tar -xf ${tar(`${version}.tar.xz`)}`.cwd(build());
        await configureMakeMakeInstall(build(version), rootfs(), env_autoconf(), [
            '-disable-mpers'
        ])
    }
}
