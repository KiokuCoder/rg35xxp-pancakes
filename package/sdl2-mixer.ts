import {$} from "bun"
import {cmakeBuild} from "../scripts/helper.ts";
import type {Pack} from "../scripts/pkg.ts";

const version = 'SDL_mixer-release-2.6.3'
export const SDLMixer: Pack = {
    name: "sdl2-mixer",
    version: "2.6.3",
    description: "SDL mixer",
    enable: false,
    dependencies: [],
    async sync({download}): Promise<void> {
        return download({
            url: `https://github.com/libsdl-org/SDL_mixer/archive/refs/tags/release-2.6.3.tar.gz`,
            filename: `SDL_mixer-release-2.6.3.tar.gz`
        })
    },
    async make({rootfs, tar, build}): Promise<void> {
        await $`[ ! -e ${build(version)} ] || rm -r ${build(version)}`.cwd(build());
        await $`tar -xf ${tar(`${version}.tar.gz`)}`.cwd(build());
        await $`external/download.sh`.cwd(build(version));
        await cmakeBuild(build(version), rootfs(), {}, [
            "-DSDL2MIXER_VENDORED=ON"
        ])
    }
}