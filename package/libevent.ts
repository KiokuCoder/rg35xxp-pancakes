import {appendFile} from "node:fs/promises";
import {$} from "bun"
import {cmakeArgs, cmakeBuild, configureMakeMakeInstall, mesonBuild, mesonCrossFile, v} from "../scripts/helper.ts";
import type {Pack} from "../scripts/pkg.ts";
import {join} from "path";

const version = 'libevent-release-2.1.12-stable';
export const libevent: Pack = {
    name: "libevent",
    version: "2.1.12-stable",
    description: "libevent is an asynchronous event notification library",
    enable: true,
    dependencies: [],
    async sync({download}): Promise<void> {
        return download({
            url: `https://github.com/libevent/libevent/archive/refs/tags/release-2.1.12-stable.tar.gz`,
            filename: `libevent-release-2.1.12-stable.tar.gz`
        })
    },
    async make({rootfs, tar, build, env_autoconf}): Promise<void> {
        await $`[ ! -e ${build(version)} ] || rm -r ${build(version)}`.cwd(build());
        await $`tar -xf ${tar(`${version}.tar.gz`)}`.cwd(build());
        await cmakeBuild(build(version), rootfs(), {}, [])
    }
}