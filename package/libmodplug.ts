import type {Pack} from "../scripts/pkg.ts";
import {$} from "bun";
import {cmakeBuild, configureMakeMakeInstall} from "../scripts/helper.ts";

export const libmodplug: Pack = {
    name: "libmodplug",
    version: "0.0.1",
    description: "",
    enable: false,
    dependencies: [],
    async sync({download, build}): Promise<void> {
        return Promise.resolve();
    },
    async make({rootfs, tar, build, env_autoconf}): Promise<void> {
        await $`[ ! -e libmodplug ] || rm -r libmodplug`.cwd(build());
        await $`git clone https://github.com/Konstanty/libmodplug.git`.cwd(build());
        const wd = build("libmodplug");
        await cmakeBuild(wd, rootfs(), {}, [
            "-DBUILD_SHARED_LIBS=ON"
        ])
    }
}

//
export const libmpg123: Pack = {
    name: "libmpg123",
    version: "0.0.1",
    description: "",
    enable: false,
    dependencies: [],
    async sync({download, build}): Promise<void> {
        return Promise.resolve();
    },
    async make({rootfs, tar, build, env_autoconf}): Promise<void> {
        await $`[ ! -e libmpg123 ] || rm -r libmpg123 `.cwd(build());
        await $`git clone https://github.com/gypified/libmpg123.git`.cwd(build());
        const wd = build("libmpg123");
        await configureMakeMakeInstall(wd, rootfs(), env_autoconf(), [])
    }
}
