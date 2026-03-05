import type {Pack} from "../scripts/pkg.ts";
import {$} from "bun";

const info = 'https://raw.githubusercontent.com/libretro/libretro-super/master/dist/info/mgba_libretro.info';
export const libretroMgba: Pack = {
    name: "libretro-mgba",
    version: "1.0.0",
    description: "",
    enable: false,
    dependencies: [],
    async sync({download}): Promise<void> {
    },
    async make({rootfs, tar, build, env_autoconf,}): Promise<void> {
        const wd = build('libretro-mgba');
        await $`[ ! -e ${wd} ] || rm -r ${wd}`.cwd(build());
        await $`git clone --depth 1 https://github.com/libretro/mgba.git libretro-mgba`.cwd(build());
        await $`make -j16 -f Makefile.libretro`.env(env_autoconf()).cwd(wd);
        await $`install -Dm 0755 mgba_libretro.so ${rootfs()}/usr/share/libretro/cores/mgba_libretro.so`.cwd(wd);
        await $`wget -O ${rootfs()}/usr/share/libretro/cores/mgba_libretro.info ${info}`

        await $`install -Dm 0755 mgba_libretro.so ${rootfs()}/root/.config/retroarch/cores/mgba_libretro.so`.cwd(wd);
        await $`wget -O ${rootfs()}/root/.config/retroarch/cores/mgba_libretro.info ${info}`
    }
}
