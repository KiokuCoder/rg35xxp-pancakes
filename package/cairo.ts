import {$} from "bun"
import {mesonBuild, v} from "../scripts/helper.ts";
import type {Pack} from "../scripts/pkg.ts";

const version = 'cairo-1.18.0';
export const cairo: Pack = {
    name: "cairo",
    version: "1.18.0",
    description: "Cairo is a 2D graphics library with support for multiple output devices.",
    enable: false,
    dependencies: [
        {
            name: "zlib",
            version: "*",
            required: true
        },
        {
            name: "pixman",
            version: "*",
            required: true
        },
        {
            name: "libpng",
            version: "*",
            required: true
        },
        {
            name: "fontconfig",
            version: "*",
            required: true
        },
        v("freetype2")
    ],
    async sync({download}): Promise<void> {
        return download({
            url: `https://www.cairographics.org/releases/${version}.tar.xz`,
            filename: `${version}.tar.xz`
        })
    },
    async make({resource, rootfs, tar, build, env_autoconf}): Promise<void> {
        await $`[ ! -e ${build(version)} ] || rm -r ${build(version)}`.cwd(build());
        await $`tar -xf ${tar(`${version}.tar.xz`)}`.cwd(build());
        await mesonBuild(build(version), rootfs(), [
            '-Dtests=disabled',
        ])
    }
}
