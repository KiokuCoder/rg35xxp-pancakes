import {$} from "bun"
import type {Pack} from "../scripts/pkg.ts";
import {mesonBuild} from "../scripts/helper.ts";


const version = 'mpv-0.36.0';
export const mpv: Pack = {
    name: "mpv",
    version: "0.36.0",
    description: "mpv is a free and open-source media player.",
    enable: false,
    dependencies: [
        {name: "ffmpeg", version: "*", required: true},
        {name: "alsa", version: "*", required: true},
        {name: "egl", version: "*", required: true},
        {name: "lua", version: "*", required: true},
        {name: "zlib", version: "*", required: true},
        {name: "libjpeg", version: "*", required: true},
        {name: "libass", version: "*", required: true},
    ],
    async sync({download}): Promise<void> {
        return download({
            url: `https://github.com/mpv-player/mpv/archive/refs/tags/v${version.split('-')[1]}.tar.gz`,
            filename: `${version}.tar.gz`
        })
    },
    async make({rootfs, tar, build, env_autoconf,}): Promise<void> {
        await $`[ ! -e ${build(version)} ] || rm -r ${build(version)}`.cwd(build());
        await $`tar -xf ${tar(`${version}.tar.gz`)}`.cwd(build());

        await mesonBuild(build(version), rootfs(), [
            '-Dwayland=disabled',
            '-Degl=enabled',
        ])
    }
}
