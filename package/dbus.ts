import {$} from "bun"
import {configureMakeMakeInstall} from "../scripts/helper.ts";
import type {Pack} from "../scripts/pkg.ts";


const version = 'dbus-1.14.0';
export const dbus: Pack = {
    name: "dbus",
    version: "1.14.0",
    description: "D-Bus is a message bus system.",
    enable: false,
    dependencies: [
        {
            name: "zlib",
            version: "*",
            required: true
        },
        {
            name: "expat",
            version: "*",
            required: true
        }
    ],
    async sync({download}): Promise<void> {
        return download({
            url: `https://dbus.freedesktop.org/releases/dbus/${version}.tar.xz`,
            filename: `${version}.tar.xz`
        })
    },
    async make({resource, rootfs, tar, build, env_autoconf}): Promise<void> {
        await $`[ ! -e ${build(version)} ] || rm -r ${build(version)}`.cwd(build());
        await $`tar -xf ${tar(`${version}.tar.xz`)}`.cwd(build());
        await configureMakeMakeInstall(build(version), rootfs(), env_autoconf(), [
            '--disable-systemd',
            '--enable-qt-help=no',
            '--with-systemdsystemunitdir=/lib/systemd/system',
            '--disable-tests'
        ])
    }
}
