import {$} from "bun"
import {mesonBuild, mesonCrossFile} from "../scripts/helper.ts";
import type {Pack} from "../scripts/pkg.ts";

const version = 'glib-2.72.3';
export const glib: Pack = {
    name: "glib",
    version: "2.72.3",
    description: "GLib provides the core application building blocks for libraries and applications written in C.",
    enable: false,
    dependencies: [],
    async sync({download}): Promise<void> {
        return download({
            url: `https://download.gnome.org/sources/glib/2.72/${version}.tar.xz`,
            filename: `${version}.tar.xz`
        })
    },
    async make({resource, rootfs, tar, build, env_autoconf}): Promise<void> {
        await $`[ ! -e ${build(version)} ] || rm -r ${build(version)}`.cwd(build());
        await $`tar -xf ${tar(`${version}.tar.xz`)}`.cwd(build());
        const args = [
            '-Dselinux=disabled',
            '-Dlibelf=disabled',
            '-Dlibmount=disabled'
        ];
        await Bun.write(build(version, "cross_file.txt"), mesonCrossFile(rootfs()));
        await $`meson setup --prefix=/usr builddir ${args} --cross-file cross_file.txt`.cwd(build(version));
        await $`meson compile -C builddir -j16`.cwd(build(version));
        await $`DESTDIR=${rootfs()} meson install -C builddir`.cwd(build(version));
    }
}
