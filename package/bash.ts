import {$} from "bun"
import {configureMakeMakeInstall} from "../scripts/helper.ts";
import type {Pack} from "../scripts/pkg.ts";

const version = 'bash-5.2.15';
export const bash: Pack = {
    name: "bash",
    version: "5.2.15",
    description: "GNU Bash is a Unix shell and command language interpreter.",
    enable: true,
    dependencies: [],
    async sync({download}): Promise<void> {
        return download({
            url: `https://ftp.gnu.org/gnu/bash/bash-${version.split('-')[1]}.tar.gz`,
            filename: `${version}.tar.gz`
        })
    },
    async make({resource, rootfs, tar, build, env_autoconf}): Promise<void> {
        await $`[ ! -e ${build(version)} ] || rm -r ${build(version)}`.cwd(build());
        await $`tar -xf ${tar(`${version}.tar.gz`)}`.cwd(build());

        await configureMakeMakeInstall(build(version), rootfs(), env_autoconf(), [
            "--without-bash-malloc"
        ])
        await $`mkdir -p ${rootfs('bin')}`
        await $`ln -s ../usr/bin/bash ${rootfs('bin/bash')}`
        await $`ln -s ../usr/bin/bash ${rootfs('bin/sh')}`
    }
}