import { $ } from "bun"
import { configureMakeMakeInstall } from "../scripts/helper.ts";
import type { Pack } from "../scripts/pkg.ts";

export const dropbear: Pack = {
    name: "dropbear",
    version: "2025.89",
    description: "",
    enable: true,
    dependencies: [
        {
            name: "libxcrypt",
            version: "4.4.36",
            required: true
        }
    ],
    async sync({ download }): Promise<void> {
        return download({
            url: `https://github.com/mkj/dropbear/archive/refs/tags/DROPBEAR_2025.89.tar.gz`,
            filename: `dropbear-DROPBEAR_2025.89.tar.gz`
        })
    },
    async make({ resource, rootfs, tar, build, env_autoconf }): Promise<void> {
        const version = 'dropbear-DROPBEAR_2025.89';
        await $`[ ! -e ${build(version)} ] || rm -r ${build(version)}`.cwd(build());
        await $`tar -xf ${tar('dropbear-DROPBEAR_2025.89.tar.gz')}`.cwd(build());

        await configureMakeMakeInstall(build(version), rootfs(), env_autoconf(), [])
        await $`mkdir -p ${rootfs()}/etc/dropbear`
    }
}
