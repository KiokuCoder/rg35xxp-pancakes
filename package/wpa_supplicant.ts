import {$} from "bun"
import type {Pack} from "../scripts/pkg.ts";
import {v} from "../scripts/helper.ts";

const version = 'wpa_supplicant-2.10';
export const wpa_supplicant: Pack = {
    name: "wpa_supplicant",
    version: "2.10",
    description: "",
    enable: true,
    dependencies: [
        {
            name: "openssl",
            version: "*",
            required: true
        },
        v("libnl"),
        v("dbus"),
    ],
    async sync({download}): Promise<void> {
        return download({
            url: `https://w1.fi/releases/${version}.tar.gz`,
            filename: `${version}.tar.gz`
        })
    },
    async make({resource, rootfs, tar, build, env_autoconf}): Promise<void> {
        await $`[ ! -e ${build(version)} ] || rm -r ${build(version)}`.cwd(build());
        await $`tar -xf ${tar(`${version}.tar.gz`)}`.cwd(build());

        const env = {
            ...process.env,
            PKG_CONFIG_PATH: "",
            PKG_CONFIG_LIBDIR: `${rootfs()}/usr/lib/pkgconfig:${rootfs()}/usr/share/pkgconfig`,
            PKG_CONFIG_SYSROOT_DIR: `${rootfs()}`
        };
        await $`cp defconfig .config`.cwd(build(version, "wpa_supplicant"));
        await $`make -j16 CC=aarch64-linux-gnu-gcc PKG_CONFIG=aarch64-linux-gnu-pkg-config`
            .env(env)
            .cwd(build(version, "wpa_supplicant"));
        await $`make install DESTDIR=${rootfs()}`.env(env_autoconf()).cwd(build(version, "wpa_supplicant"));
    }
}
