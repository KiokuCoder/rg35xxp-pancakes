import {$} from "bun"
import type {Pack} from "../scripts/pkg.ts";


const version = 'evtest-evtest-1.35';
export const evtest: Pack = {
    name: "evtest",
    version: "1.35.0",
    description: "Input device event monitor and query tool",
    enable: false,
    dependencies: [],
    async sync({download}): Promise<void> {
        return download({
            url: `https://gitlab.freedesktop.org/libevdev/evtest/-/archive/evtest-1.35/evtest-evtest-1.35.tar.gz`,
            filename: `${version}.tar.gz`
        })
    },
    async make({output,rootfs, tar, resource,build, env_autoconf}): Promise<void> {
        const CC = output('aarch64-linux-gnu/bin/aarch64-linux-gnu-gcc');
        await $`[ ! -e ${build(version)} ] || rm -r ${build(version)}`.cwd(build());
        await $`tar -xf ${tar(`${version}.tar.gz`)}`.cwd(build());
        await $`${CC} -o evtest evtest.c`.cwd(build(version));
        await $`install evtest ${rootfs()}/usr/bin/evtest`.cwd(build(version));
        await $`${CC} ${resource('gamepad_test.c')} -o gamepad_test -lSDL2 -lSDL2_ttf -L${rootfs('usr/lib64')} -I${rootfs('usr/include')}`.cwd(build(version));
        await $`install gamepad_test ${rootfs()}/usr/bin/gamepad_test`.cwd(build(version));
    }
}