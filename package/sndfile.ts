import {$} from "bun"
import {cmakeArgs} from "../scripts/helper.ts";
import type {Pack} from "../scripts/pkg.ts";

const version = 'libsndfile-1.2.2';
export const libsndfile: Pack = {
    name: "libsndfile",
    version: "1.2.2",
    description: "sndfile is a C library for reading and writing files containing sampled sound (such as WAV and AIFF files).",
    enable: false,
    dependencies: [],
    async sync({download}): Promise<void> {
        return download({
            url: `https://github.com/libsndfile/libsndfile/archive/refs/tags/1.2.2.tar.gz`,
            filename: `${version}.tar.gz`
        })
    },
    async make({resource, rootfs, tar, build, env_autoconf}): Promise<void> {
        await $`[ ! -e ${build(version)} ] || rm -r ${build(version)}`.cwd(build());
        await $`tar -xf ${tar(`${version}.tar.gz`)}`.cwd(build());
        await $`mkdir build`.cwd(build(version));
        await $`cmake -DCMAKE_POSITION_INDEPENDENT_CODE=ON ${cmakeArgs(rootfs())} ..`.cwd(build(version, 'build'));
        await $`make -j16`.cwd(build(version, 'build'));
        await $`make install DESTDIR=${rootfs()}`.cwd(build(version, 'build'));
    }
}
