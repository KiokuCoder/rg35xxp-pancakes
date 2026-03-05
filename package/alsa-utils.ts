import {$} from "bun"
import {configureMakeMakeInstall} from "../scripts/helper.ts";
import type {Pack} from "../scripts/pkg.ts";


const version = 'alsa-utils-1.2.12';
export const alsaUtils: Pack = {
    name: "alsa-utils",
    version: "1.2.12",
    description: "The Advanced Linux Sound Architecture (ALSA) provides audio and MIDI functionality to the Linux operating system.",
    enable: false,
    dependencies: [
        {
            name: "alsa",
            version: "*",
            required: true
        },
        {
            name: "ncurses",
            version: "*",
            required: true
        },
        {
            name: "udev",
            version: "*",
            required: true
        }
    ],
    async sync({download}): Promise<void> {
        return download({
            url: `https://www.alsa-project.org/files/pub/utils/${version}.tar.bz2`,
            filename: `${version}.tar.bz2`
        })
    },
    async make({resource, rootfs, tar, build, env_autoconf}): Promise<void> {
        await $`[ ! -e ${build(version)} ] || rm -r ${build(version)}`.cwd(build());
        await $`tar -xf ${tar(`${version}.tar.bz2`)}`.cwd(build());
        await configureMakeMakeInstall(build(version), rootfs(), env_autoconf(), [
            '--disable-nls'
            // '--disable-xmlto','--disable-rst2man'
            // '--disable-alsamixer'
        ])
    }
}