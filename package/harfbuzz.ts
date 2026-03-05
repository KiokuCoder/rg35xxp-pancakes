import {$} from "bun"
import type {Pack} from "../scripts/pkg.ts";
import {mesonBuild} from "../scripts/helper.ts";


const version = 'harfbuzz-10.1.0';
export const harfbuzz: Pack = {
    name: "harfbuzz",
    version: "9.0.0",
    description: "HarfBuzz is an OpenType text shaping library.",
    enable: true,
    dependencies: [],
    async sync({download}): Promise<void> {
        return download({
            url: `https://github.com/harfbuzz/harfbuzz/archive/refs/tags/${version.split('-')[1]}.tar.gz`,
            filename: `${version}.tar.gz`
        })
    },
    async make({rootfs, tar, build,}): Promise<void> {
        await $`[ ! -e ${build(version)} ] || rm -r ${build(version)}`.cwd(build());
        await $`tar -xf ${tar(`${version}.tar.gz`)}`.cwd(build());
        await mesonBuild(build(version), rootfs(), [])
    }
}
