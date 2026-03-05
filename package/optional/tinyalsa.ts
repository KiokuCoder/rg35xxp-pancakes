import {$} from "bun"
import type {Pack} from "../../scripts/pkg.ts";

const version = 'tinyalsa-2.0.0';
export const tinyalsa: Pack = {
    name: "tinyalsa",
    version: "2.0.0",
    description: "Tinyalsa is a small library to interface with ALSA in the Linux kernel.",
    enable: false,
    dependencies: [],
    async sync({download}): Promise<void> {
        return download({
            url: `https://github.com/tinyalsa/tinyalsa/archive/refs/tags/v${version.split('-')[1]}.tar.gz`,
            filename: `${version}.tar.gz`
        })
    },
    async make({resource, rootfs, tar, build, env_autoconf}): Promise<void> {
        await $`[ ! -e ${build(version)} ] || rm -r ${build(version)}`.cwd(build());
        await $`tar -xf ${tar(`${version}.tar.gz`)}`.cwd(build());
        await $`make CROSS_COMPILE=aarch64-linux-gnu- -j16`.cwd(build(version));
        await $`make install PREFIX=/usr DESTDIR=${rootfs()}`.cwd(build(version));
    }
}
