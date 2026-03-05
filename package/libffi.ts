import {$} from "bun"
import {configureMakeMakeInstall} from "../scripts/helper.ts";
import type {Pack} from "../scripts/pkg.ts";

const version = 'libffi-3.4.4';
export const libffi: Pack = {
    name: "libffi",
    version: "3.4.4",
    description: "A Portable Foreign Function Interface Library.",
    enable: false,
    dependencies: [],
    async sync({download}): Promise<void> {
        return download({
            url: `https://github.com/libffi/libffi/releases/download/v${version.split('-')[1]}/${version}.tar.gz`,
            filename: `${version}.tar.gz`
        })
    },
    async make({resource, rootfs, tar, build, env_autoconf}): Promise<void> {
        await $`[ ! -e ${build(version)} ] || rm -r ${build(version)}`.cwd(build());
        await $`tar -xf ${tar(`${version}.tar.gz`)}`.cwd(build());
        await configureMakeMakeInstall(build(version), rootfs(), env_autoconf(), [])
    }
}
