import {$} from "bun"
import type {Pack} from "../scripts/pkg.ts";
import {configureMakeMakeInstall} from "../scripts/helper.ts";


const version = 'gmp-6.2.1';
export const gmp: Pack = {
    name: "gmp",
    version: "6.2.1",
    description: "GMP is a free library for arbitrary precision arithmetic, operating on signed integers, rational numbers, and floating-point numbers.",
    enable: false,
    dependencies: [],
    async sync({download}): Promise<void> {
        return download({
            url: `https://gmplib.org/download/gmp/${version}.tar.xz`,
            filename: `${version}.tar.xz`
        })
    },
    async make({rootfs, tar, build, env_autoconf}): Promise<void> {
        await $`[ ! -e ${build(version)} ] || rm -r ${build(version)}`.cwd(build());
        await $`tar -xf ${tar(`${version}.tar.xz`)}`.cwd(build());
        await configureMakeMakeInstall(build(version), rootfs(), env_autoconf(), [])
    }
}
