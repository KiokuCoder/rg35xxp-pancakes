import {$} from "bun"
import {configureMakeMakeInstall, v} from "../../scripts/helper.ts";
import type {Pack} from "../../scripts/pkg.ts";

const version = 'DirectFB-1.7.7';
export const directfb: Pack = {
    name: "directfb",
    version: "1.7.7",
    description: "DirectFB is a graphics library.",
    enable: false,
    dependencies: [
        v("libpng"),
        v("libjpeg"),
        v("freetype2")
    ],
    async sync({download}): Promise<void> {
        return download({
            url: `https://sources.buildroot.net/directfb/DirectFB-1.7.7.tar.gz`,
            filename: `${version}.tar.gz`
        })
    },
    async make({resource, rootfs, tar, build, env_autoconf}): Promise<void> {
        await $`[ ! -e ${build(version)} ] || rm -r ${build(version)}`.cwd(build());
        await $`tar -xf ${tar(`${version}.tar.gz`)}`.cwd(build());

        await configureMakeMakeInstall(build(version), rootfs(), env_autoconf(), [

            "--enable-debug",
            "--enable-trace",

            // "--without-software",
            "--with-inputdrivers=none",
            "--with-gfxdrivers=gles2",

            "--enable-egl",
            "--enable-fbdev",
            "--disable-sdl",
            "--disable-vnc",
            "--disable-drmkms",
        ])
    }
}
