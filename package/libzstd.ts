import {$} from "bun"
import type {Pack} from "../scripts/pkg.ts";

const version = 'zstd-1.5.6';
export const zstd: Pack = {
    name: "zstd",
    version: "1.5.6",
    description: "",
    enable: false,
    dependencies: [],
    async sync({download}): Promise<void> {
        return download({
            url: `https://github.com/facebook/zstd/releases/download/v${version.split('-')[1]}/${version}.tar.gz`,
            filename: `${version}.tar.gz`,
        })
    },
    async make({resource, rootfs, tar, build, env_autoconf}): Promise<void> {
        await $`[ ! -e ${build(version)} ] || rm -r ${build(version)}`.cwd(build());
        await $`tar -xf ${tar(`${version}.tar.gz`)}`.cwd(build());
        await $`make -j16`.env(env_autoconf()).cwd(build(version));
        await $`make install PREFIX=/usr DESTDIR=${rootfs()}`.env(env_autoconf()).cwd(build(version));
    }
}
