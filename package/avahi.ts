import {$} from "bun"
import type {Pack} from "../scripts/pkg.ts";


const version = 'avahi-0.8';
export const avahi: Pack = {
    name: "avahi",
    version: "0.8",
    description: "Avahi client libraries",
    enable: true,
    dependencies: [
        {
            name: "libdaemon",
            version: "*",
            required: true
        },
        {
            name: "libevent",
            version: "*",
            required: true
        },
        {
            name: "dbus",
            version: "*",
            required: true
        },
        {
            name: "glib",
            version: "*",
            required: true
        },
    ],
    async sync({download}): Promise<void> {
        await download({
            url: `https://github.com/avahi/avahi/archive/refs/tags/v0.8.tar.gz`,
            filename: `avahi-0.8.tar.gz`
        })
    },
    async make({rootfs, tar, build, env_autoconf}): Promise<void> {
        await $`[ ! -e ${build(`${version}`)} ] || rm -r ${build(`${version}`)}`.cwd(build());
        await $`tar -xf ${tar(`${version}.tar.gz`)}`.cwd(build());

        const args = [
            '--prefix=/usr',
            '--host=aarch64-linux-gnu',
            "--disable-static",
            "--disable-nls",
            "--disable-gtk",
            "--disable-gtk3",
            "--disable-qt5",
            "--disable-qt4",
            '--disable-glib',
            "--disable-gobject",
            "--disable-python",
            "--disable-mono",
            "--disable-monodoc",
            "--disable-gdbm",
            "--with-distro=none",
            "--with-systemdsystemunitdir=/lib/systemd/system",
            "--enable-compat-libdns_sd",
            "--disable-manpages",
            "--disable-xmltoman",
            "--disable-autoipd",
        ];
        await $`./autogen.sh ${args}`.env(env_autoconf()).cwd(build(version))
        await $`make -j16`.env(env_autoconf()).cwd(build(version));
        await $`make install DESTDIR=${rootfs()}`.env(env_autoconf()).cwd(build(version));
    }
}