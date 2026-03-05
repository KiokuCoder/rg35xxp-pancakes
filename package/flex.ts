import {$} from "bun"
import type {Pack} from "../scripts/pkg.ts";

const version = 'flex-2.6.4';
export const flex: Pack = {
    name: "flex",
    version: "2.6.4",
    description: "Flex is a tool for generating scanners which recognize lexical patterns in text.",
    enable: false,
    dependencies: [],
    async sync({download}): Promise<void> {
        return download({
            url: `https://github.com/westes/flex/releases/download/v2.6.4/${version}.tar.gz`,
            filename: `${version}.tar.gz`
        })
    },
    async make({resource, rootfs, tar, build, env_autoconf}): Promise<void> {
        await $`[ ! -e ${build(version)} ] || rm -r ${build(version)}`.cwd(build());
        await $`tar -xf ${tar(`${version}.tar.gz`)}`.cwd(build());

        const args = [
            '--host=aarch64-linux',
            '--prefix=/usr',
            '--disable-doc',
            '--disable-nls'
        ];

        await $`./configure ${args}`.env(env_autoconf()).cwd(build(version));
        await $`make -j16`.env(env_autoconf({CFLAGS: `-D_LARGEFILE_SOURCE -D_LARGEFILE64_SOURCE -D_FILE_OFFSET_BITS=64  -O2 -g0 -D_FORTIFY_SOURCE=1`})).cwd(build(version));
        await $`make install DESTDIR=${rootfs()}`.cwd(build(version));
    }
}
