import type {Pack} from "../scripts/pkg.ts";
import {$} from "bun";


const version = 'u-boot';
export const uboot: Pack = {
    name: "u-boot",
    version: "2024.10",
    description: "Das U-Boot -- the Universal Boot Loader",
    enable: true,
    dependencies: [],
    async sync({download}): Promise<void> {
        return download({
            url: `https://github.com/u-boot/u-boot/archive/refs/tags/v2024.10.tar.gz`,
            filename: `u-boot-2024.10.tar.gz`
        })
    },
    async make({rootfs, tar, build, env_autoconf}): Promise<void> {
        // await $`[ ! -e ${build(version)} ] || rm -r ${build(version)}`.cwd(build());
        // await $`tar -xf ${tar(`${version}.tar.gz`)}`.cwd(build());

        const defconfig = "anbernic_rg35xx_h700_defconfig"; // Replace with your desired defconfig
        const cross_compile = "aarch64-linux-gnu-"; // Replace with your desired cross-compiler prefix

        await $`make ${defconfig} CROSS_COMPILE=${cross_compile}`.cwd(build(version));
        await $`make CROSS_COMPILE=${cross_compile}`.env({
            ...process.env,
            BL31:build('trusted-firmware-a/build/sun50i_h616/debug/bl31.bin')
        }).cwd(build(version));

        // u-boot doesn't typically install to a rootfs like other packages.
        // Instead, the build process generates various u-boot images (e.g., u-boot.bin, u-boot.img)
        // that are then flashed to the target device. 
        // You might want to copy these images to a specific location.
        // For example:
        // await $`cp ${build(version)}/u-boot.bin ${rootfs('boot')}`

    }
}