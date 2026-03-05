import {$} from "bun"
import {configureMakeMakeInstall} from "../scripts/helper.ts";
import type {Pack} from "../scripts/pkg.ts";

const version = 'file-5.39';
export const file: Pack = {
    name: "file",
    version: "5.39",
    description: "Determine file type with magic numbers.",
    enable: false,
    dependencies: [],
    async sync({download}): Promise<void> {
        return download({
            url: `https://astron.com/pub/file/${version}.tar.gz`,
            filename: `${version}.tar.gz`
        })
    },
    async make({resource, rootfs, tar, build, env_autoconf}): Promise<void> {
        await $`[ ! -e ${build(version)} ] || rm -r ${build(version)}`.cwd(build());
        await $`tar -xf ${tar(`${version}.tar.gz`)}`.cwd(build());
        await configureMakeMakeInstall(build(version), rootfs(), env_autoconf({
            ac_cv_prog_file_magic: "yes"
        }), [
            '--disable-bzlib', // 需要 bzip2 支持
            '--disable-xzlib',  // 需要 xz 支持
        ])
    }
}
