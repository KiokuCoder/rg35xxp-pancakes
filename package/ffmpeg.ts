import {$} from "bun"
import type {Pack} from "../scripts/pkg.ts";

const version = 'ffmpeg-6.0';
export const ffmpeg: Pack = {
    name: "ffmpeg",
    version: "6.0",
    description: "A complete, cross-platform solution to record, convert and stream audio and video.",
    enable: false,
    dependencies: [
        {
            name: "x264",
            version: "*",
            required: true
        },
        {
            name: "x265",
            version: "*",
            required: true
        },
        {
            name: "opencl",
            version: "*",
            required: true
        }
    ],
    async sync({download}): Promise<void> {
        return download({
            url: `https://ffmpeg.org/releases/${version}.tar.xz`,
            filename: `${version}.tar.xz`
        })
    },
    async make({resource, rootfs, tar, build, env_autoconf}): Promise<void> {
        await $`[ ! -e ${build(version)} ] || rm -r ${build(version)}`.cwd(build());
        await $`tar -xf ${tar(`${version}.tar.xz`)}`.cwd(build());

        const args = [
            `--prefix=/usr`,
            `--arch=aarch64`,
            `--target-os=linux`,
            `--cross-prefix=aarch64-linux-gnu-`,
            `--sysroot=${rootfs()}`,
            `--enable-cross-compile`,
            `--disable-debug`,
            `--disable-doc`,
            `--enable-shared`,
            `--disable-static`,
            `--enable-zlib`,
            `--enable-gpl`,
            `--enable-nonfree`,
            `--enable-libx264`,
            `--enable-libx265`,
            `--enable-opencl`
        ];

        await $`./configure ${args}`.env(env_autoconf()).cwd(build(version));
        await $`make -j16`.cwd(build(version));
        await $`make install DESTDIR=${rootfs()}`.cwd(build(version));
    }
};
