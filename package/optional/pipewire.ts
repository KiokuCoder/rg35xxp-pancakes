import {$} from "bun"
import type {Pack} from "../../scripts/pkg.ts";
import {mesonBuild, v} from "../../scripts/helper.ts";

const version = 'pipewire-1.2.6';
export const pipewire: Pack = {
    name: "pipewire",
    version: "1.2.6",
    description: "PipeWire is a server and user space API to deal with multimedia pipelines.",
    enable: false,
    dependencies: [
        v("dbus")
    ],
    async sync({download}): Promise<void> {
        return download({
            url: `https://github.com/PipeWire/pipewire/archive/refs/tags/1.2.6.tar.gz`,
            filename: `${version}.tar.gz`
        });
    },
    async make({resource, rootfs, tar, build, env_autoconf}): Promise<void> {
        await $`[ ! -e ${build(version)} ] || rm -r ${build(version)}`.cwd(build());
        await $`tar -xf ${tar(`${version}.tar.gz`)}`.cwd(build());
        await mesonBuild(build(version), rootfs(), [
            '-Dtests=disabled',
        ])
    }
}
