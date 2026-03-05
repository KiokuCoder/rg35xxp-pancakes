import {$} from "bun"
import {configureMakeMakeInstall, v} from "../scripts/helper.ts";
import type {Pack} from "../scripts/pkg.ts";

const version = 'tmux-3.6a';
export const tmux: Pack = {
    name: "tmux",
    version: "3.6a",
    description: "tmux is a terminal multiplexer",
    enable: false,
    dependencies: [
        v("ncurses"),
        v("libevent")
    ],
    async sync({download}): Promise<void> {
        return download({
            url: `https://github.com/tmux/tmux/releases/download/${version.split('-')[1]}/${version}.tar.gz`,
            filename: `${version}.tar.gz`
        })
    },
    async make({resource, rootfs, tar, build, env_autoconf}): Promise<void> {
        await $`[ ! -e ${build(version)} ] || rm -r ${build(version)}`.cwd(build());
        await $`tar -xf ${tar(`${version}.tar.gz`)}`.cwd(build());
        await configureMakeMakeInstall(build(version), rootfs(), env_autoconf(), [])
    }
}
