import {$} from "bun"
import type {Pack} from "../scripts/pkg.ts";

const version = 'ncurses-6.4';
export const ncurses: Pack = {
    name: "ncurses",
    version: "6.4",
    description: "Ncurses is a library that provides an API for character-based user interfaces.",
    enable: false,
    dependencies: [],
    async sync({download}): Promise<void> {
        return download({
            url: `https://ftp.gnu.org/gnu/ncurses/${version}.tar.gz`,
            filename: `${version}.tar.gz`
        })
    },
    async make({rootfs, tar, build, env_autoconf}): Promise<void> {

        await $`[ ! -e ${build(version)} ] || rm -r ${build(version)}`.cwd(build());
        await $`tar -xf ${tar(`${version}.tar.gz`)}`.cwd(build());
        const args = [
            'CC=aarch64-linux-gnu-gcc',
            '--prefix=/usr',
            '--host=aarch64-linux-gnu',
            '--with-shared',
            '--with-normal',
            '--enable-pc-files',
            '--with-pkg-config-libdir=/usr/lib/pkgconfig',
            '--without-ada',
            '--without-manpages',
            '--without-debug',
            '--without-tests',
            '--without-progs'
        ];
        await $`./configure ${args}`.cwd(build(version));
        await $`make -j16`.cwd(build(version));
        await $`make install DESTDIR=${rootfs()}`.cwd(build(version));

        await $`make clean`.cwd(build(version));
        await $`./configure ${args} --enable-widec`.cwd(build(version));
        await $`make -j16`.cwd(build(version));
        await $`make install DESTDIR=${rootfs()}`.cwd(build(version));
    }
};
