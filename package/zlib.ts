import {$} from "bun"
import type {Pack} from "../scripts/pkg.ts";


const version = 'zlib-1.3.2';
export const zlib: Pack = {
    name: "zlib",
    version: "1.3.2",
    description: "zlib is a software library used for data compression as well as a data format.",
    enable: false,
    dependencies: [],
    async sync({download}): Promise<void> {
        return download({
            url: `https://github.com/madler/zlib/releases/download/v${version.split('-')[1]}/${version}.tar.gz`,
            filename: `${version}.tar.gz`
        })
    },
    async make({rootfs, tar, build, env_autoconf}): Promise<void> {
        await $`[ ! -e ${build(version)} ] || rm -r ${build(version)}`.cwd(build());
        await $`tar -xf ${tar(`${version}.tar.gz`)}`.cwd(build());
        const env = {
            ...process.env,
            CC: "aarch64-linux-gnu-gcc",
            AR: "aarch64-linux-gnu-ar",
            RANLIB: "aarch64-linux-gnu-ranlib",
        }
        await $`./configure --prefix=/usr`.env(env).cwd(build(version));
        await $`make -j16`.cwd(build(version));
        await $`make install DESTDIR=${rootfs()}`.cwd(build(version));
    }
}
