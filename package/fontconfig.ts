import {$} from "bun"
import {configureMakeMakeInstall, v} from "../scripts/helper.ts";
import type {Pack} from "../scripts/pkg.ts";

const version = 'fontconfig-2.14.2';
export const fontconfig: Pack = {
    name: "fontconfig",
    version: "2.14.2",
    description: "Fontconfig is a library designed to locate and configure fonts.",
    enable: false,
    dependencies: [
        v("freetype2"), v("expat"),
        {
            name: "libxml2",
            version: "*",
            required: false
        }
    ],
    async sync({download}): Promise<void> {
        return download({
            url: `https://www.freedesktop.org/software/fontconfig/release/${version}.tar.gz`,
            filename: `${version}.tar.gz`
        })
    },
    async make({resource, rootfs, tar, build, env_autoconf}): Promise<void> {
        await $`[ ! -e ${build(version)} ] || rm -r ${build(version)}`.cwd(build());
        await $`tar -xf ${tar(`${version}.tar.gz`)}`.cwd(build());
        await configureMakeMakeInstall(build(version), rootfs(), env_autoconf({}), [
            '--without-python'
        ])
    },
}
