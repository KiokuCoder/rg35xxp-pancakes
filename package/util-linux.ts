import {$} from "bun"
import {configureMakeMakeInstall, mesonBuild, v} from "../scripts/helper.ts";
import type {Pack} from "../scripts/pkg.ts";

const version = "util-linux-2.38";
export const utils: Pack = {
    name: "util-linux",
    version: "2.38",
    description: "",
    enable: true,
    dependencies: [],
    async sync({download}): Promise<void> {
        return download({
            url: `https://cdn.kernel.org/pub/linux/utils/util-linux/v${version.split('-')[2]}/${version}.tar.gz`,
            filename: `${version}.tar.gz`,
        })
    },
    async make({resource, rootfs, tar, build, env_autoconf}): Promise<void> {
        await $`[ ! -e ${build(version)} ] || rm -r ${build(version)}`.cwd(build());
        await $`tar -xf ${tar(`${version}.tar.gz`)}`.cwd(build());
        await configureMakeMakeInstall(build(version), rootfs(), env_autoconf(), [])
    }
}
export const kmod: Pack = {
    name: "kmod",
    version: "34.2",
    description: "",
    enable: true,
    dependencies: [v("openssl"),v("zlib")],
    async sync({download}): Promise<void> {
        return download({
            url: `https://www.kernel.org/pub/linux/utils/kernel/kmod/kmod-34.2.tar.gz`,
            filename: `kmod-34.2.tar.gz`,
        })
    },
    async make({resource, rootfs, tar, build, env_autoconf}): Promise<void> {
        const tarball = tar('kmod-34.2.tar.gz');
        const wd = build('kmod-34.2');
        await $`[ ! -e ${wd} ] || rm -r ${wd}`.cwd(build());
        await $`tar -xf ${tarball}`.cwd(build());
        await mesonBuild(wd, rootfs(), [
            "-Dzstd=disabled",
            "-Dxz=disabled",
            "-Dmanpages=false",
            "-Ddocs=false",
        ])
    }
}
export const procps: Pack = {
    name: "procps",
    version: "4.06",
    description: "",
    enable: true,
    dependencies: [v("ncurses")],
    async sync({download}): Promise<void> {
        return download({
            url: `https://gitlab.com/procps-ng/procps/-/archive/v4.0.6/procps-v4.0.6.tar.gz`,
            filename: `procps-v4.0.6.tar.gz`,
        })
    },
    async make({resource, rootfs, tar, build, env_autoconf}): Promise<void> {
        const tarball = tar('procps-v4.0.6.tar.gz');
        const wd = build('procps-v4.0.6');
        await $`[ ! -e ${wd} ] || rm -r ${wd}`.cwd(build());
        await $`tar -xf ${tarball}`.cwd(build());
        await $`./autogen.sh`.cwd(wd);
        await configureMakeMakeInstall(wd, rootfs(), env_autoconf(),[])
    }
}

