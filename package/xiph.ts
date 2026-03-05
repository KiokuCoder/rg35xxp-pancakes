import {$} from "bun"
import type {Pack} from "../scripts/pkg.ts";
import {cmakeBuild, configureMakeMakeInstall} from "../scripts/helper.ts";

const version = 'libvorbis-1.3.7';

export const vorbis: Pack = {
    name: "vorbis",
    version: "1.3.7",
    description: "Ogg Vorbis is a fully open, non-proprietary, patent-and-royalty-free, general-purpose compressed audio format.",
    enable: false,
    dependencies: [
        {name: "libogg", version: "*", required: true}
    ],
    provide: [
        {
            name: "libvorbis",
            version: "1.3.7",
        },
        {
            name: "libvorbisfile",
            version: "1.3.7",
        },
    ],
    async sync({download}): Promise<void> {
        return download({
            url: `https://github.com/xiph/vorbis/releases/download/v${version.split('-')[1]}/${version}.tar.gz`,
            filename: `${version}.tar.gz`
        })
    },
    async make({rootfs, tar, build, env_autoconf}): Promise<void> {
        await $`[ ! -e ${build(version)} ] || rm -r ${build(version)}`.cwd(build());
        await $`tar -xf ${tar(`${version}.tar.gz`)}`.cwd(build());
        await cmakeBuild(build(version), rootfs(), {}, [
            "-DBUILD_SHARED_LIBS=ON"
        ])
    }
}

export const ogg: Pack = {
    name: "ogg",
    version: "1.3.6",
    description: "",
    enable: false,
    dependencies: [],
    provide: [
        {
            name: "libogg",
            version: "1.3.6",
        }
    ],
    async sync({download}): Promise<void> {
        return download({
            url: `https://github.com/xiph/ogg/releases/download/v1.3.6/libogg-1.3.6.tar.gz`,
            filename: `libogg-1.3.6.tar.gz`
        })
    },
    async make({rootfs, tar, build, env_autoconf}): Promise<void> {
        const version = 'libogg-1.3.6';
        await $`[ ! -e ${build(version)} ] || rm -r ${build(version)}`.cwd(build());
        await $`tar -xf ${tar(`${version}.tar.gz`)}`.cwd(build());
        await cmakeBuild(build(version), rootfs(), {}, [
            "-DCMAKE_POSITION_INDEPENDENT_CODE=ON",
            "-DBUILD_SHARED_LIBS=ON"
        ])
    }
}

export const theora: Pack = {
    name: "theora",
    version: "1.2.0",
    description: "",
    enable: false,
    dependencies: [],
    provide: [
        {
            name: "libtheora",
            version: "1.2.0",
        }
    ],
    async sync({download}): Promise<void> {
        return download({
            url: `https://github.com/xiph/theora/archive/refs/tags/v1.2.0.tar.gz`,
            filename: `theora-1.2.0.tar.gz`
        })
    },
    async make({rootfs, tar, build, env_autoconf}): Promise<void> {
        const version = 'theora-1.2.0';
        await $`[ ! -e ${build(version)} ] || rm -r ${build(version)}`.cwd(build());
        await $`tar -xf ${tar(`${version}.tar.gz`)}`.cwd(build());
        await $`./autogen.sh`.cwd(build(version)).env(env_autoconf());
        await configureMakeMakeInstall(build(version), rootfs(), env_autoconf(), [])
    }
}