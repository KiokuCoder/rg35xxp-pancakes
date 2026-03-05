import {$} from "bun"
import type {Pack} from "../scripts/pkg.ts";


const version = 'x264-snapshot-20191217-2245-stable';
export const x264: Pack = {
    name: "x264",
    version: "20191217",
    description: "A free and open-source software library and a command-line utility developed by VideoLAN for encoding video streams into the H.264/MPEG-4 AVC format.",
    enable: false,
    dependencies: [],
    async sync({download}): Promise<void> {
        return download({
            url: `https://download.videolan.org/pub/videolan/x264/snapshots/${version}.tar.bz2`,
            filename: `${version}.tar.bz2`
        })
    },
    async make({resource, rootfs, tar, build, env_autoconf}): Promise<void> {
        await $`[ ! -e ${build(version)} ] || rm -r ${build(version)}`.cwd(build());
        await $`tar -xf ${tar(`${version}.tar.bz2`)}`.cwd(build());
        const env = {
            ...process.env,
            PKG_CONFIG_PATH: "",
            PKG_CONFIG_LIBDIR: `${rootfs()}/usr/lib/pkgconfig:${rootfs()}/usr/share/pkgconfig`,
            PKG_CONFIG_SYSROOT_DIR: `${rootfs()}`
        };

        const args = [
            '--host=aarch64-linux-gnu',
            `--cross-prefix=aarch64-linux-gnu-`,
            `--prefix=/usr`,
            `--sysroot=${rootfs()}`,
            `--enable-shared`,
            `--enable-pic`,
            `--disable-cli`
        ];

        await $`./configure ${args}`.env(env).cwd(build(version));
        await $`make -j16`.cwd(build(version));
        await $`make install DESTDIR=${rootfs()}`.cwd(build(version));
    },
};
