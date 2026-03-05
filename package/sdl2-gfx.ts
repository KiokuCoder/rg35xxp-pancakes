import {$, env} from "bun"
import {cmakeBuild, configureMakeMakeInstall} from "../scripts/helper.ts";
import type {Pack} from "../scripts/pkg.ts";

const version = 'SDL2_gfx';
export const SDLGfx: Pack = {
    name: "sdl2-gfx",
    version: "0.0.1",
    description: "SDL image",
    enable: false,
    dependencies: [],
    async sync({download}): Promise<void> {
    },
    async make({installed, rootfs, resource,output, tar, build, env_autoconf}): Promise<void> {
        const wd = build(version);
        await $`[ ! -e ${build(version)} ] || rm -r ${build(version)}`.cwd(build());
        await $`git clone --depth 1 https://github.com/ferzkopp/SDL2_gfx.git`.cwd(build());
        const args = [
            `-I${rootfs()}/usr/include`,
            `-I${rootfs()}/usr/include/SDL2`,
            `-L${rootfs()}/usr/lib`,
            '-lSDL2',
            'SDL2_framerate.c', 'SDL2_gfxPrimitives.c', 'SDL2_imageFilter.c', 'SDL2_rotozoom.c',
            '-o', 'libSDL2_gfx.so', '-shared', '-fPIC'
        ];
        await $`${output('aarch64-linux-gnu/bin/aarch64-linux-gnu-gcc')} ${args}`.cwd(wd);
        const headers = [
            'SDL2_framerate.h', 'SDL2_gfxPrimitives.h', 'SDL2_gfxPrimitives_font.h', 'SDL2_imageFilter.h', 'SDL2_rotozoom.h'
        ]
        await $`install -m0755 ${headers} ${rootfs()}/usr/include/SDL2`.cwd(wd);
        await $`install -m0756 libSDL2_gfx.so ${rootfs()}/usr/lib`.cwd(wd);
        await $`install -m0755 ${resource('SDL2_gfx.pc')} ${rootfs()}/usr/lib/pkgconfig`.cwd(wd);
    },
}
