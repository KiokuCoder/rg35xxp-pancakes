import { $ } from "bun"
import {cmakeBuild, configureMakeMakeInstall, v} from "../../scripts/helper.ts";
import type { Pack } from "../../scripts/pkg.ts";

// 可以通过编译，但是无法正常启动游戏，需要打补丁
const version = 'love-11.5';
export const love2d: Pack = {
    name: "love2d",
    version: "11.5",
    description: "LÖVE is an awesome 2D game framework for Lua.",
    enable: false,
    dependencies: [
        v("sdl2"),
        v("luajit"),
        v("freetype2"),
        v("zlib"),
        v("libpng"),
        v("openal"),
        v("libmodplug"),
        v("libvorbis"),
        v("libvorbisfile"),
        v("libtheora"),
        v("libmpg123"),
    ],
    async sync({ download }): Promise<void> {
        return download({
            url: `https://github.com/love2d/love/releases/download/11.5/${version}-linux-src.tar.gz`,
            filename: `${version}.tar.gz`
        })
    },
    async make({ rootfs, tar, build, env_autoconf }): Promise<void> {
        await $`[ ! -e ${build(version)} ] || rm -r ${build(version)}`.cwd(build());
        await $`tar -xf ${tar(`${version}.tar.gz`)}`.cwd(build());
        await configureMakeMakeInstall(build(version), rootfs(), env_autoconf(), [])
    }
}
