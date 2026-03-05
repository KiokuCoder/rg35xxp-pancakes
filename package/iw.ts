import {$} from "bun";
import type {Pack} from "../scripts/pkg.ts";

const version = 'iw-6.9';
export const iw: Pack = {
    name: "iw",
    version: "6.9",
    description: "iw is a new nl80211 based CLI configuration utility for wireless devices.",
    enable: true,
    dependencies: [
        {
            name: "libnl",
            version: "*",
            required: true
        }
    ],
    async sync({download}): Promise<void> {
        return download({
            url: `https://www.kernel.org/pub/software/network/iw/${version}.tar.xz`,
            filename: `${version}.tar.xz`
        })
    },
    async make({rootfs, tar, build, env_autoconf}): Promise<void> {
        // 清理并解压
        await $`[ ! -e ${build(version)} ] || rm -r ${build(version)}`.cwd(build());
        await $`tar -xf ${tar(`${version}.tar.xz`)}`.cwd(build());

        const env = env_autoconf();

        await $`make -j$(nproc)`.cwd(build(version)).env(env);

        // 安装到 rootfs
        await $`make install DESTDIR=${rootfs()} SBINDIR=/sbin`.cwd(build(version)).env(env);
    }
}