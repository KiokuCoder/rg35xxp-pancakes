import {$} from "bun"
import type {Pack} from "../scripts/pkg.ts";
import {configureMakeMakeInstall} from "../scripts/helper.ts";


const version = 'readline-8.1';
export const readline: Pack = {
    name: "readline",
    version: "8.1",
    description: "The GNU Readline library provides a set of functions for use by applications that allow users to edit command lines as they are typed in.",
    enable: false,
    dependencies: [],
    async sync({download}): Promise<void> {
        return download({
            url: `https://ftp.gnu.org/gnu/readline/${version}.tar.gz`,
            filename: `${version}.tar.gz`
        })
    },
    async make({rootfs, tar, build, env_autoconf}): Promise<void> {
        await $`[ ! -e ${build(version)} ] || rm -r ${build(version)}`.cwd(build());
        await $`tar -xf ${tar(`${version}.tar.gz`)}`.cwd(build());
        await configureMakeMakeInstall(build(version), rootfs(), env_autoconf(), ['CFLAGS=-fPIC'])
    }
}
