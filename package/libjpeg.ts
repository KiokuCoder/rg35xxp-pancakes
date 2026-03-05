import {$} from "bun"
import {cmakeArgs} from "../scripts/helper.ts";
import type {Pack} from "../scripts/pkg.ts";

const version = 'libjpeg-turbo-2.1.4';
export const libjpeg: Pack = {
    name: "libjpeg",
    version: "2.1.4",
    description: "",
    enable: false,
    dependencies: [],
    async sync({download}): Promise<void> {
        return download({
            url: `https://download.sourceforge.net/libjpeg-turbo/${version}.tar.gz`,
            filename: `${version}.tar.gz`
        })
    },
    async make({resource, rootfs, tar, build, env_autoconf}): Promise<void> {
        await $`[ ! -e ${build(version)} ] || rm -r ${build(version)}`.cwd(build());
        await $`tar -xf ${tar(`${version}.tar.gz`)}`.cwd(build());

        await $`mkdir build`.cwd(build(version));
        await $`cmake -G"Unix Makefiles" -DENABLE_STATIC=OFF ${cmakeArgs(rootfs())} ..`.cwd(build(version, 'build'));
        await $`make -j16`.cwd(build(version, 'build'));
        await $`make install DESTDIR=${rootfs()}`.cwd(build(version, 'build'));

    }
}
