import {$} from "bun"
import type {Pack} from "../scripts/pkg.ts";
import {configureMakeMakeInstall} from "../scripts/helper.ts";


const version = 'nettle-3.8.1';
export const nettle: Pack = {
    name: "nettle",
    version: "3.8.1",
    description: "Nettle is a cryptographic library that is designed to fit easily in more or less any context.",
    enable: false,
    dependencies: [
        {
            name: "gmp",
            version: "*",
            required: true,
        }
    ],
    async sync({download}): Promise<void> {
        return download({
            url: `https://ftp.gnu.org/gnu/nettle/${version}.tar.gz`,
            filename: `${version}.tar.gz`
        })
    },
    async make({rootfs, tar, build, env_autoconf}): Promise<void> {
        await $`[ ! -e ${build(version)} ] || rm -r ${build(version)}`.cwd(build());
        await $`tar -xf ${tar(`${version}.tar.gz`)}`.cwd(build());
        await configureMakeMakeInstall(build(version), rootfs(), env_autoconf(), [])
    }
}
