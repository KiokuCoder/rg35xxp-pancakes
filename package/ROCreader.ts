import type {Pack} from "../scripts/pkg.ts";
import {$} from "bun";
import {v} from "../scripts/helper.ts";

export const ROCreader: Pack = {
    name: "ROCreader",
    version: "0.1.0",
    description: "A comic reader used in models 34xx and 34xxsp based on the H700 chip.",
    enable: false,
    dependencies: [
        v("libzip"),
        v("sdl2-ttf"),
        v("sdl2-image"),
        {name: "sdl2", version: "*", required: true},
    ],
    async sync({download}): Promise<void> {
    },
    async make({rootfs, output, tar, build, env_autoconf,}): Promise<void> {
        const wd = build('ROCreader');
        await $`[ ! -e ${wd} ] || rm -r ${wd}`.cwd(build());
        await $`git clone https://github.com/LPF970915/ROCreader.git`.cwd(build());
        await $`make print-config`.env(env_autoconf()).cwd(wd);
        await $`make`.env(env_autoconf()).cwd(wd);
    }
}
