import {$} from "bun"
import type {Pack} from "../scripts/pkg.ts";

const version = 'pcre-8.45';
export const pcre: Pack = {
    name: "pcre",
    version: "8.45",
    description: "Perl Compatible Regular Expressions library.",
    enable: false,
    dependencies: [],
    async sync({download}): Promise<void> {
        return download({
            url: `https://zenlayer.dl.sourceforge.net/project/pcre/pcre/8.45/${version}.tar.gz`,
            filename: `${version}.tar.gz`
        })
    },
    async make({resource, rootfs, tar, build, env_autoconf}): Promise<void> {
        await $`[ ! -e ${build(version)} ] || rm -r ${build(version)}`.cwd(build());
        await $`tar -xf ${tar(`${version}.tar.gz`)}`.cwd(build());
        const args = [
            '--host=aarch64-linux-gnu',
            '--prefix=/usr',
            '--disable-nls' //Disable Native Language Support
        ];
        await $`./configure ${args}`.cwd(build(version));
        await $`make -j16`.cwd(build(version));
        await $`make install DESTDIR=${rootfs()}`.cwd(build(version));
    }
}