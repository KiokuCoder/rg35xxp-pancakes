import {$} from "bun"
import type {Pack} from "../scripts/pkg.ts";
import {mesonBuild, v} from "../scripts/helper.ts";


const version = 'libass-0.17.2';
export const libass: Pack = {
    name: "libass",
    version: "0.17.2",
    description: "Libass is a portable subtitle renderer for the ASS/SSA (Advanced SubStation Alpha/SubStation Alpha) subtitle format.",
    enable: false,
    dependencies: [v("fribidi"), v("harfbuzz"), v("freetype2"), v("fontconfig")],
    async sync({download}): Promise<void> {
        return download({
            url: `https://github.com/libass/libass/archive/refs/tags/0.17.2.tar.gz`,
            filename: `${version}.tar.xz`
        })
    },
    async make({rootfs, tar, build, env_autoconf}): Promise<void> {
        await $`[ ! -e ${build(version)} ] || rm -r ${build(version)}`.cwd(build());
        await $`tar -xf ${tar(`${version}.tar.xz`)}`.cwd(build());
        await mesonBuild(build(version), rootfs(), [])
    }
}
