import {$, env} from "bun"
import {cmakeBuild, configureMakeMakeInstall} from "../scripts/helper.ts";
import type {Pack} from "../scripts/pkg.ts";

const version = 'SDL_ttf-release-2.22.0'
export const SDLTTF: Pack = {
    name: "sdl2-ttf",
    version: "2.8.2",
    description: "SDL ttf",
    enable: false,
    dependencies: [],
    async sync({download}): Promise<void> {
        return download({
            url: `https://github.com/libsdl-org/SDL_ttf/archive/refs/tags/release-2.22.0.tar.gz`,
            filename: `${version}.tar.gz`,
        })
    },
    async make({installed, rootfs, resource, tar, build, env_autoconf}): Promise<void> {
        await $`[ ! -e ${build(version)} ] || rm -r ${build(version)}`.cwd(build());
        await $`tar -xf ${tar(`${version}.tar.gz`)}`.cwd(build());
        await cmakeBuild(build(version), rootfs(), {}, [])
    },
}
