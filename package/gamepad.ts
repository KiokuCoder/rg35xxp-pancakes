import type {Pack} from "../scripts/pkg.ts";
import {$} from "bun";
import {v} from "../scripts/helper.ts";

export const gamepad: Pack = {
    name: "gamepad",
    version: "1.0.0",
    description: "",
    enable: false,
    dependencies: [v("tinyalsa")],
    async sync({download}): Promise<void> {
    },
    async make({rootfs, resource, build, env_autoconf}): Promise<void> {

        const audio = build('audio');
        await $`[ ! -e ${audio} ] || rm -r ${audio}`;
        await $`tar -xf ${resource(`audio.tar.gz`)}`.cwd(build());
        await $`make -j16`.env(env_autoconf()).cwd(audio);
        await $`install -m 0755 audio ${rootfs('bin/audio')}`.cwd(audio);

        const src = build('gamepad');
        await $`[ ! -e ${src} ] || rm -r ${src}`;
        await $`tar -xf ${resource(`gamepad.tar.gz`)}`.cwd(build());
        await $`make -j16`.env(env_autoconf()).cwd(src);
        await $`install -m 0755 gamepad ${rootfs('bin/gamepad')}`.cwd(src);
    },
}
