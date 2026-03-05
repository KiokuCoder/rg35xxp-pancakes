import {$} from "bun"
import type {Pack} from "../scripts/pkg.ts";

const version = 'icu-release-76-1';
export const icu: Pack = {
    name: "icu",
    version: "74.2",
    description: "ICU is a mature, widely used set of C/C++ and Java libraries providing Unicode and Globalization support for software applications.",
    enable: true,
    dependencies: [],
    async sync({download}): Promise<void> {
        return download({
            url: `https://github.com/unicode-org/icu/archive/refs/tags/release-76-1.tar.gz`,
            filename: `icu-release-76-1.tar.gz`
        })
    },
    async make({rootfs, tar, build, env_autoconf}): Promise<void> {
        await $`[ ! -e ${build(version)} ] || rm -r ${build(version)}`.cwd(build());
        await $`tar -xf ${tar(`${version}.tar.gz`)}`.cwd(build());
        const host = build(version, `icu4c/source/host`);
        await $`mkdir -p ${host}`.cwd(build());
        await $`../configure`.cwd(host);
        await $`make -j16`.cwd(host);

        const wd = build(version, `icu4c/source`);
        const args0 = [
            '--prefix=/usr',
            '--host=aarch64-linux-gnu',
            `--with-cross-build=${host}`,
            '--disable-static',
            '--enable-shared',
            `--disable-tests`,
            `--disable-samples`
        ]
        await $`./configure ${args0}`.env(env_autoconf()).cwd(wd);
        await $`make -j16`.env(env_autoconf()).cwd(wd);
        await $`make install DESTDIR=${rootfs()}`.env(env_autoconf()).cwd(wd);
    }
}