import {$, env} from "bun"
import {cmakeBuild, configureMakeMakeInstall} from "../scripts/helper.ts";
import type {Pack} from "../scripts/pkg.ts";

const version = 'SDL_image-release-2.8.2'
export const SDLImage: Pack = {
    name: "sdl2-image",
    version: "2.8.2",
    description: "SDL image",
    enable: false,
    dependencies: [],
    async sync({download}): Promise<void> {
        return download({
            url: `https://codeload.github.com/libsdl-org/SDL_image/tar.gz/refs/tags/release-2.8.2`,
            filename: `SDL_image-release-2.8.2.tar.gz`
        })
    },
    async make({installed, rootfs, resource, tar, build, env_autoconf}): Promise<void> {
        await $`[ ! -e ${build(version)} ] || rm -r ${build(version)}`.cwd(build());
        await $`tar -xf ${tar(`${version}.tar.gz`)}`.cwd(build());
        await cmakeBuild(build(version), rootfs(), {}, [])
    },
}
