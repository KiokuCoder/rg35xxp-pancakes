import {$, env} from "bun"
import {cmakeBuild, configureMakeMakeInstall, v} from "../scripts/helper.ts";
import type {Pack} from "../scripts/pkg.ts";

const version = 'SDL2-2.32.10';
export const SDL2: Pack = {
    name: "sdl2",
    version: "2.32.10",
    description: "Simple DirectMedia Layer is a cross-platform development library designed to provide low level access to audio, keyboard, mouse, joystick, and graphics hardware via OpenGL and Direct3D.",
    enable: false,
    dependencies: [
        {
            name: "alsa",
            version: "*",
            required: true
        },
        {
            name: "egl",
            version: "*",
            required: true
        }
    ],
    async sync({download}): Promise<void> {
        return download({
            url: `https://github.com/libsdl-org/SDL/releases/download/release-${version.split('-')[1]}/${version}.tar.gz`,
            filename: `${version}.tar.gz`
        })
    },
    async make({installed, rootfs, resource, tar, build, env_autoconf}): Promise<void> {
        await $`[ ! -e ${build(version)} ] || rm -r ${build(version)}`.cwd(build());
        await $`tar -xf ${tar(`${version}.tar.gz`)}`.cwd(build());

        await $`patch -p1 < ${resource('patch/SDL2-2.32.10.patch')}`.cwd(build(version))

        await cmakeBuild(build(version), rootfs(), {}, [
            "-DSDL_WAYLAND=OFF",
            "-DSDL_X11=OFF",
            "-DSDL_PIPEWIRE=OFF",
            "-DSDL_PULSEAUDIO=OFF",
            "-DSDL_JACK=OFF",
            "-DSDL_OPENGL=OFF",
            "-DSDL_OPENGLES=ON",
            "-DSDL_DBUS=OFF",
            "-DSDL_IBUS=OFF"
        ])
    },
}

export const SDL2ALL: Pack = {
    name: "sdl2-all",
    version: "0.0.1",
    description: "",
    enable: false,
    dependencies: [
        v("sdl2"),
        v("sdl2-gfx"),
        v("sdl2-image"),
        v("sdl2-ttf"),
        v("sdl2-mixer"),
    ],
    async sync(): Promise<void> {
    },
    async make(): Promise<void> {
    },
}
