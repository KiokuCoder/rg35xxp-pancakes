import type {Pack} from "../../scripts/pkg.ts";
import {$} from "bun";
import {cmakeBuild} from "../../scripts/helper.ts";

// 可以通过编译，但是找不到可以运行的游戏
export const krkrsdl2: Pack = {
    name: "krkrsdl2",
    version: "1.8.9",
    description: "",
    enable: false,
    dependencies: [
        {name: "sdl2", version: "*", required: true},
    ],
    async sync({download}): Promise<void> {
    },
    async make({rootfs, tar, resource, build, env_autoconf}): Promise<void> {
        const wd = build("krkrsdl2-git");
        await $`[ ! -e ${wd} ] || rm -r ${wd}`.cwd(build());
        await $`git clone https://github.com/krkrsdl2/krkrsdl2.git krkrsdl2-git`.cwd(build());
        await $`git checkout efc5923d86cd762706d819e8fa6425faa3c23f79`.cwd(wd);
        await $`git submodule update --init --recursive`.cwd(wd);
        await $`patch -p1 < ${resource('patch/krkrsdl2_SDL2.patch')}`.cwd(wd)
        await cmakeBuild(wd, rootfs(), {}, [])
    }
}
