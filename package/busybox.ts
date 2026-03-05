import {$} from "bun"
import type {Pack} from "../scripts/pkg.ts";

const version = 'busybox-1.36.1';
export const busybox: Pack = {
    name: "busybox",
    version: "1.36.1",
    description: "BusyBox is a software suite that provides several Unix utilities in a single executable file.",
    enable: true,
    dependencies: [
        {
            name: "zlib",
            version: "> 1.0.0",
            required: true
        }
    ],
    async sync({download}): Promise<void> {
        return download({
            url: `https://busybox.net/downloads/${version}.tar.bz2`,
            filename: `${version}.tar.bz2`
        })
    },
    async make({rootfs, tar, build}): Promise<void> {
        await $`[ ! -e ${build(version)} ] || rm -r ${build(version)}`.cwd(build());
        await $`tar -xf ${tar(`${version}.tar.bz2`)}`.cwd(build());
        const args = ["-j16", "CROSS_COMPILE=aarch64-linux-gnu-"]
        await $`make ${args} defconfig`.cwd(build(version));
        await $`sed -i 's/# CONFIG_USE_PORTABLE_CODE is not set/CONFIG_USE_PORTABLE_CODE=y/' .config`.cwd(build(version));

        // 不安装软链接
        await $`sed -i 's/# CONFIG_INSTALL_APPLET_DONT is not set/CONFIG_INSTALL_APPLET_DONT=y/' .config`.cwd(build(version));
        await $`sed -i 's/CONFIG_INSTALL_APPLET_SYMLINKS=y/CONFIG_INSTALL_APPLET_SYMLINKS=n/' .config`.cwd(build(version));
        await $`make ${args} `.cwd(build(version));
        await $`make ${args} install CONFIG_PREFIX=${rootfs()}`.cwd(build(version));
    }
}
