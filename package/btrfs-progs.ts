import {$} from "bun"
import type {Pack} from "../scripts/pkg.ts";

const version = "btrfs-progs-v6.9";
export const btrfsprogs: Pack = {
    name: "btrfsprogs",
    version: "6.9",
    description: "",
    enable: false,
    dependencies: [],
    async sync({download}): Promise<void> {
        return download({
            url: `https://cdn.kernel.org/pub/linux/kernel/people/kdave/btrfs-progs/btrfs-progs-v6.9.tar.xz`,
            filename: `${version}.tar.xz`
        })
    },
    async make({resource, rootfs, tar, build, env_autoconf}): Promise<void> {
        await $`[ ! -e ${build(version)} ] || rm -r ${build(version)}`.cwd(build());
        await $`tar -xf ${tar(`${version}.tar.xz`)}`.cwd(build());
        const env = {
            ...process.env,
            PKG_CONFIG_PATH: "",
            PKG_CONFIG_LIBDIR: `${rootfs()}/usr/lib/pkgconfig:${rootfs()}/usr/share/pkgconfig`,
            PKG_CONFIG_SYSROOT_DIR: `${rootfs()}`
        }
        const args = [
            '--host=aarch64-linux-gnu',
            '--prefix=/usr',
            '--disable-documentation',
            '--disable-python',
            '--disable-libblkid',
        ];
        await $`./configure ${args}`.env(env).cwd(build(version));
        await $`make -j16`.cwd(build(version));
        await $`make install DESTDIR=${rootfs()}`.cwd(build(version));
    }
}
