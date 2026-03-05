import {$} from "bun"
import type {Pack} from "../scripts/pkg.ts";
import {configureMakeMakeInstall} from "../scripts/helper.ts";


const version = 'connman-1.43';
export const connman: Pack = {
    name: "connman",
    version: "1.43",
    description: "Connection manager daemon for managing internet connections",
    enable: false,
    dependencies: [
        {
            name: "iptables",
            version: "*",
            required: true
        },
        {
            name: "gnutls",
            version: "*",
            required: true
        },
        {
            name: "readline",
            version: "*",
            required: true
        },
        {
            name: "ncurses",
            version: "*",
            required: true
        },
    ],
    async sync({download}): Promise<void> {
        await download({
            url: `https://www.kernel.org/pub/linux/network/connman/${version}.tar.xz`,
            filename: `${version}.tar.xz`,
        });
    },
    async make({rootfs, tar, build, env_autoconf}): Promise<void> {
        await $`[ ! -e ${build(version)} ] || rm -r ${build(version)}`.cwd(build());
        await $`tar -xf ${tar(`${version}.tar.xz`)}`.cwd(build());
        await configureMakeMakeInstall(build(version), rootfs(), env_autoconf({
            LIBS: "-lncurses"
        }), []);
    }
};
