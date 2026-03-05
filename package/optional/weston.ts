import {appendFile} from "node:fs/promises";
import {$} from "bun"
import {mesonBuild, v} from "../../scripts/helper.ts";
import type {Pack} from "../../scripts/pkg.ts";


const version = 'weston-9.0.93';
export const weston: Pack = {
    name: "weston",
    version: "9.0.93",
    description: "Weston is a Wayland compositor and a useful reference implementation of the Wayland protocol.",
    enable: false,
    dependencies: [
        v("libinput"),
        v("wayland"),
        v("xkbcommon"),
        v("pixman"),
        v("libdrm"),
        v("cairo"),
        v("libjpeg"),
        v("egl"),
        v("pam")
    ],
    async sync({download}): Promise<void> {
        return download({
            url: `https://gitlab.freedesktop.org/wayland/weston/-/archive/9.0.93/weston-9.0.93.tar.gz`,
            filename: `${version}.tar.xz`
        })
    },
    async make({resource, rootfs, tar, build, env_autoconf}): Promise<void> {
        await $`[ ! -e ${build(version)} ] || rm -r ${build(version)}`.cwd(build());
        await $`tar -xf ${tar(`${version}.tar.xz`)}`.cwd(build());
        // await $`sed -i 's/deprecated-backend-fbdev/backend-fbdev/g' meson_options.txt`.cwd(build(version))
        await appendFile(build(version, 'meson_options.txt'), `option(
\t'backend-fbdev',
\ttype: 'boolean',
\tvalue: true
)`)
        // await $`sed -i 's/deprecated-backend-fbdev/backend-fbdev/g' meson_options.txt`.cwd(build(version))
        await mesonBuild(build(version), rootfs(), [
            '-Dsystemd=false',
            '-Dbackend-drm=false',
            '-Dbackend-drm-screencast-vaapi=false',
            '-Dbackend-headless=false',
            '-Dbackend-rdp=false',
            '-Dscreenshare=false',
            '-Dbackend-wayland=false',
            '-Dbackend-x11=false',
            '-Ddeprecated-backend-fbdev=true',
            '-Dbackend-fbdev=true',
            '-Dbackend-default=fbdev',
            '-Dremoting=false',
            '-Dxwayland=false',
            '-Dpipewire=false',
            '-Dlauncher-logind=false',
            '-Dlauncher-libseat=false',
            '-Dcolor-management-lcms=false',
            '-Dcolor-management-colord=false',
            '-Dsimple-clients=egl',
            '-Ddemo-clients=false',
            '-Dimage-webp=false',
            '-Ddoc=false'
        ]);
    }
}