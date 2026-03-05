import {$} from "bun"
import {configureMakeMakeInstall} from "../scripts/helper.ts";
import type {Pack} from "../scripts/pkg.ts";

const version = "eudev-3.2.14";
export const eudev: Pack = {
    name: "eudev",
    version: "3.2.14",
    description: "",
    enable: false,
    provide: [{
        name: "udev",
        version: "3.2.14",
    }],
    dependencies: [],
    async sync({download}): Promise<void> {
        return download({
            url: `https://github.com/eudev-project/eudev/releases/download/v${version.split("-")[1]}/${version}.tar.gz`,
            filename: `${version}.tar.gz`,
        })
    },
    async make({rootfs, tar, build, env_autoconf}): Promise<void> {
        await $`[ ! -e ${build(version)} ] || rm -r ${build(version)}`.cwd(build());
        await $`tar -xf ${tar(`${version}.tar.gz`)}`.cwd(build());
        await configureMakeMakeInstall(build(version), rootfs(), env_autoconf(), [])
    },
}
