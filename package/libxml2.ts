import {$} from "bun"
import {configureMakeMakeInstall} from "../scripts/helper.ts";
import type {Pack} from "../scripts/pkg.ts";

const version = 'libxml2-2.11.5';
export const libxml2: Pack = {
    name: "libxml2",
    version: "2.11.5",
    description: "libxml2 is a software library for parsing XML documents.",
    enable: false,
    dependencies: [],
    async sync({download}): Promise<void> {
        return download({
            url: `https://download.gnome.org/sources/libxml2/2.11/${version}.tar.xz`,
            filename: `${version}.tar.xz`
        })
    },
    async make({resource, rootfs, tar, build, env_autoconf}): Promise<void> {
        await $`[ ! -e ${build(version)} ] || rm -r ${build(version)}`.cwd(build());
        await $`tar -xf ${tar(`${version}.tar.xz`)}`.cwd(build());

        await configureMakeMakeInstall(build(version), rootfs(), env_autoconf(), [
            '--without-python',
        ])
    }
}