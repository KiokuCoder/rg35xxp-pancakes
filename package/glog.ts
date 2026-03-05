import type {Pack} from "../scripts/pkg.ts";
import {$} from "bun";
import {cmakeBuild} from "../scripts/helper.ts";

export const glog: Pack = {
    name: "glog",
    version: "0.7.1",
    description: "Google logging library",
    enable: false,
    dependencies: [],
    async sync({download}): Promise<void> {
        return download({
            url: `https://github.com/google/glog/archive/refs/tags/v0.7.1.tar.gz`,
            filename: `glog-0.7.1.tar.gz`
        })
    },
    async make({rootfs, tar, build, env_autoconf}): Promise<void> {
        await $`[ ! -e glog-0.7.1 ] || rm -r glog-0.7.1`.cwd(build());
        await $`tar -xf ${tar(`glog-0.7.1.tar.gz`)}`.cwd(build());
        await cmakeBuild(build('glog-0.7.1'), rootfs(), {}, ["-DWITH_GTEST=OFF", "-DWITH_PKGCONFIG=ON", "-DBUILD_EXAMPLES=OFF"])
    }
}
