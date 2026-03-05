import {$} from "bun"
import {mesonBuild, v} from "../../scripts/helper.ts";
import type {Pack} from "../../scripts/pkg.ts";

const version = 'wayland-1.22.0';
export const wayland: Pack = {
    name: "wayland",
    version: "1.22.0",
    description: "Wayland is a protocol for a compositor to talk to its clients as well as a C library implementation of that protocol.",
    enable: false,
    dependencies: [v("wayland-protocols"), v("libffi"), v("expat"), v("libxml2")],
    async sync({download}): Promise<void> {
        return download({
            url: `https://gitlab.freedesktop.org/wayland/wayland/-/releases/${version.split('-')[1]}/downloads/${version}.tar.xz`,
            filename: `${version}.tar.xz`
        })
    },
    async make({resource, rootfs, tar, build, env_autoconf}): Promise<void> {
        await $`[ ! -e ${build(version)} ] || rm -r ${build(version)}`.cwd(build());
        await $`tar -xf ${tar(`${version}.tar.xz`)}`.cwd(build());
        await mesonBuild(build(version), rootfs(), [
            '-Dtests=false',
            '-Ddocumentation=false'
        ])
    }
}

export const wayland_protocols: Pack = {
    name: "wayland-protocols",
    version: "1.25",
    description: "",
    enable: false,
    dependencies: [],
    async sync({download}): Promise<void> {
        return download({
            url: `https://gitlab.freedesktop.org/wayland/wayland-protocols/-/archive/1.25/wayland-protocols-1.25.tar.gz`,
            filename: `wayland-protocols-1.25.tar.xz`
        })
    },
    async make({resource, rootfs, tar, build, env_autoconf}): Promise<void> {
        const version = 'wayland-protocols-1.25';
        await $`[ ! -e ${build(version)} ] || rm -r ${build(version)}`.cwd(build());
        await $`tar -xf ${tar(`${version}.tar.xz`)}`.cwd(build());
        await mesonBuild(build(version), rootfs(), [
            '-Dtests=false',
        ])
    }
}
