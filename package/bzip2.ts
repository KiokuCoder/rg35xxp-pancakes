import { $ } from "bun"
import type { Pack } from "../scripts/pkg.ts";

const version = 'bzip2-1.0.8';
export const bzip2: Pack = {
    name: "bzip2",
    version: "1.0.8",
    description: "bzip2 is a free and open-source file compression program that uses the Burrows-Wheeler algorithm.",
    enable: false,
    dependencies: [],
    async sync({ download }): Promise<void> {
        return download({
            url: `https://sourceware.org/pub/bzip2/${version}.tar.gz`,
            filename: `${version}.tar.gz`
        })
    },
    async make({ rootfs, tar, build, resource }): Promise<void> {
        await $`[ ! -e ${build(version)} ] || rm -r ${build(version)}`.cwd(build());
        await $`tar -xf ${tar(`${version}.tar.gz`)}`.cwd(build());
        const args = ["CC=aarch64-linux-gnu-gcc", "AR=aarch64-linux-gnu-ar", "RANLIB=aarch64-linux-gnu-ranlib"]
        // await $`make ${args} -j16 bzip2`.cwd(build(version));
        // await $`make ${args} -j16 bzip2recover`.cwd(build(version));

        // Compile shared library if needed
        await $`make ${args} -f Makefile-libbz2_so -j16`.cwd(build(version));

        await $`make ${args} PREFIX=${rootfs()}/usr install`.cwd(build(version));

        await $`mkdir -p ${rootfs()}/usr/lib/pkgconfig`;
        await $`install -m0755 ${resource('bzip2.pc')} ${rootfs()}/usr/lib/pkgconfig`;
    }
}
