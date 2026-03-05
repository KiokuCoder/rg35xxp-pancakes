import {$} from "bun"
import {configureMakeMakeInstall} from "../scripts/helper.ts";
import type {Pack} from "../scripts/pkg.ts";

const version = 'alsa-lib-1.2.12';
export const alsa: Pack = {
    name: "alsa",
    version: "1.2.12",
    description: "ALSA library",
    enable: true,
    dependencies: [],
    async sync({download}): Promise<void> {
        return download({
            url: `https://www.alsa-project.org/files/pub/lib/alsa-lib-1.2.12.tar.bz2`,
            filename: `${version}.tar.bz2`
        })
    },
    async make({resource, rootfs, tar, build, env_autoconf}): Promise<void> {
        await $`[ ! -e ${build(version)} ] || rm -r ${build(version)}`.cwd(build());
        await $`tar -xf ${tar(`${version}.tar.bz2`)}`.cwd(build());
        await configureMakeMakeInstall(build(version), rootfs(), env_autoconf(), [
            '--disable-python',
            '--disable-alisp',
        ])
    }
}
