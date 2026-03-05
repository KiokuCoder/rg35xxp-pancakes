import {$} from "bun"
import {configureMakeMakeInstall} from "../scripts/helper.ts";
import type {Pack} from "../scripts/pkg.ts";

const version = 'expat-2.6.3';
export const expat: Pack = {
    name: "expat",
    version: "2.6.3",
    description: "Expat is a stream-oriented XML parser library written in C.",
    enable: false,
    dependencies: [],
    async sync({download}): Promise<void> {
        return download({
            url: `https://github.com/libexpat/libexpat/releases/download/R_2_6_3/${version}.tar.xz`,
            filename: `${version}.tar.xz`
        })
    },
    async make({resource, rootfs, tar, build, env_autoconf}): Promise<void> {
        await $`[ ! -e ${build(version)} ] || rm -r ${build(version)}`.cwd(build());
        await $`tar -xf ${tar(`${version}.tar.xz`)}`.cwd(build());

        await configureMakeMakeInstall(build(version), rootfs(), env_autoconf(), [])
    }
}
