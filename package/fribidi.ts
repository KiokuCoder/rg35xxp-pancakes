import {$} from "bun"
import type {Pack} from "../scripts/pkg.ts";
import {mesonBuild} from "../scripts/helper.ts";


const version = 'fribidi-1.0.12';
export const fribidi: Pack = {
    name: "fribidi",
    version: "1.0.12",
    description: "The Free Implementation of the Unicode Bidirectional Algorithm",
    enable: true,
    dependencies: [],
    async sync({download}): Promise<void> {
        return download({
            url: `https://github.com/fribidi/fribidi/archive/refs/tags/v1.0.12.tar.gz`,
            filename: `${version}.tar.xz`
        })
    },
    async make({rootfs, tar, build, env_autoconf}): Promise<void> {
        await $`[ ! -e ${build(version)} ] || rm -r ${build(version)}`.cwd(build());
        await $`tar -xf ${tar(`${version}.tar.xz`)}`.cwd(build());
        await mesonBuild(build(version), rootfs(), [
            '-Ddocs=false'
        ])
    }
}
