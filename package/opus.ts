import type {Pack} from "../scripts/pkg.ts";
import {configureMakeMakeInstall, v} from "../scripts/helper.ts";
import {$} from "bun";


const version = 'opus-1.3';
export const opus: Pack = {
    name: "opus",
    version: "1.3",
    description: "Opus is a lossy audio coding format developed by the Xiph.Org Foundation and standardized by the Internet Engineering Task Force, designed to efficiently code speech and general audio in a single format, while remaining low-latency enough for real-time interactive communication and low-complexity enough for low-end embedded processors.",
    enable: true,
    dependencies: [
        v('util-linux'),
    ],
    async sync({download}): Promise<void> {
        return download({
            url: `https://archive.mozilla.org/pub/opus/${version}.tar.gz`,
            filename: `${version}.tar.gz`
        })
    },
    async make({rootfs, tar, build, env_autoconf}): Promise<void> {
        await $`[ ! -e ${build(version)} ] || rm -r ${build(version)}`.cwd(build());
        await $`tar -xf ${tar(`${version}.tar.gz`)}`.cwd(build());
        await configureMakeMakeInstall(build(version), rootfs(), env_autoconf(), [])
    }
}