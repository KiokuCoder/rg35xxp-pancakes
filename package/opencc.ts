import type {Pack} from "../scripts/pkg.ts";
import {cmakeBuild, v} from "../scripts/helper.ts";
import {$} from "bun";

export const opencc: Pack = {
    name: "opencc",
    version: "1.1.9",
    description: "OpenCC is a project for conversion between Traditional and Simplified Chinese",
    enable: true,
    dependencies: [],
    async sync({download}): Promise<void> {
        return download({
            url: `https://github.com/BYVoid/OpenCC/archive/refs/tags/ver.1.1.9.tar.gz`,
            filename: `OpenCC-ver.1.1.9.tar.gz`
        })
    },
    async make({rootfs, tar, build}): Promise<void> {
        const wd = build('OpenCC-ver.1.1.9');
        await $`[ ! -e OpenCC-ver.1.1.9 ] || rm -r OpenCC-ver.1.1.9`.cwd(build());
        await $`tar -xf ${tar(`OpenCC-ver.1.1.9.tar.gz`)}`.cwd(build());
        await cmakeBuild(wd, rootfs(), {}, [])
    }
}
