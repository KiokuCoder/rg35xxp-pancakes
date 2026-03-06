import type {Pack} from "../../../scripts/pkg.ts";
import {cmakeBuild, v} from "../../../scripts/helper.ts";
import {$} from "bun";

const version = '0.15.2';
export const librime: Pack = {
    name: "librime",
    version,
    description: "Rime input method engine",
    enable: true,
    dependencies: [v("yaml-cpp"), v("boost"), v("glog"), v("opencc"), v("marisa"), v("leveldb")],
    async sync({download}): Promise<void> {
        return download({
            url: `https://github.com/rime/librime/archive/refs/tags/1.12.0.tar.gz`,
            filename: `librime-1.12.0.tar.gz`
        })
    },
    async make({rootfs, tar, build, env_autoconf}): Promise<void> {
        const wd = build('librime-1.12.0');
        await $`[ ! -e librime-1.12.0 ] || rm -r librime-1.12.0`.cwd(build());
        await $`tar -xf ${tar(`librime-1.12.0.tar.gz`)}`.cwd(build());
        await cmakeBuild(wd, rootfs(), {}, ["-DBUILD_TEST=OFF"])
    }
}
