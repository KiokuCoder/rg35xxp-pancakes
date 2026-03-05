import {$} from "bun"
import {configureMakeMakeInstall} from "../scripts/helper.ts";
import type {Pack} from "../scripts/pkg.ts";


const version = 'Linux-PAM-1.5.1';
export const pam: Pack = {
    name: "pam",
    version: "1.5.1",
    description: "Pluggable Authentication Modules for Linux",
    enable: false,
    dependencies: [],
    async sync({download}): Promise<void> {
        return download({
            url: `https://github.com/linux-pam/linux-pam/releases/download/v${version.split('-')[2]}/${version}.tar.xz`,
            filename: `${version}.tar.xz`
        })
    },
    async make({resource, rootfs, tar, build, env_autoconf}): Promise<void> {
        await $`[ ! -e ${build(version)} ] || rm -r ${build(version)}`.cwd(build());
        await $`tar -xf ${tar(`${version}.tar.xz`)}`.cwd(build());
        await configureMakeMakeInstall(build(version), rootfs(), env_autoconf(), [])
    }
}
