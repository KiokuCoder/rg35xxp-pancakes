import {$} from "bun"
import {mesonBuild} from "../scripts/helper.ts";
import type {Pack} from "../scripts/pkg.ts";

const version = 'pulseaudio-16.1';
export const pulseaudio: Pack = {
    name: "pulseaudio",
    version: "16.1",
    description: "PulseAudio is a sound server for POSIX and Win32 systems.",
    enable: false,
    dependencies: [],
    async sync({download}): Promise<void> {
        return download({
            url: `https://www.freedesktop.org/software/pulseaudio/releases/${version}.tar.xz`,
            filename: `${version}.tar.xz`
        })
    },
    async make({resource, rootfs, tar, build, env_autoconf}): Promise<void> {
        await $`[ ! -e ${build(version)} ] || rm -r ${build(version)}`.cwd(build());
        await $`tar -xf ${tar(`${version}.tar.xz`)}`.cwd(build());
        await mesonBuild(build(version), rootfs(), [
            '-Dman=false',
            '-Ddatabase=simple',
            '-Dtests=false',
            '-Ddbus=disabled',
            '-Dgsettings=disabled',
            '-Dsystemd=disabled',
            '-Dbluez5=disabled',
        ])
    }
}
