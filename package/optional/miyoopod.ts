import {$} from "bun"
import type {Pack} from "../../scripts/pkg.ts";

// 可以通过编译，但是无法启动，需要打补丁
export const miyoopod: Pack = {
    name: "miyoopod",
    version: "0.1.0",
    description: "",
    enable: false,
    dependencies: [
        {name: "sdl2", version: "*", required: true},
        {name: "sdl2-mixer", version: "*", required: true},
    ],
    async sync({download}): Promise<void> {
    },
    async make({rootfs, output, tar, build, env_autoconf,}): Promise<void> {
        const wd = build('miyoopod-git');
        await $`[ ! -e ${wd} ] || rm -r ${wd}`.cwd(build());
        await $`git clone https://github.com/danfragoso/miyoopod.git miyoopod-git`.cwd(build());
        await $`git checkout 68c3d50c1bb5ae5cb71ddfddb525a7195c0777b6`.cwd(wd);
        await $`sed -i 's@#include <SDL\\.h>@#include <SDL2/SDL.h>@g' src/main.c`.cwd(wd);
        await $`sed -i 's@#include "SDL\\.h"@#include "SDL2/SDL.h"@g' src/audio.c`.cwd(wd);
        await $`sed -i 's@#include "SDL_mixer\\.h"@#include "SDL2/SDL_mixer.h"@g' src/audio.c`.cwd(wd);
        const env = {
            ...process.env,
            PKG_CONFIG_PATH: "",
            PKG_CONFIG_SYSROOT_DIR: rootfs(),
            PKG_CONFIG_LIBDIR: `${rootfs('/usr/lib64/pkgconfig')}:${rootfs('/usr/share/pkgconfig')}`,
            CC: output('aarch64-linux-gnu/bin/aarch64-linux-gnu-gcc'),
            LD: output('aarch64-linux-gnu/bin/aarch64-linux-gnu-ld'),
            GOOS: "linux",
            GOARCH: "arm64",
            CGO_ENABLED: "1",
            CGO_LDFLAGS: `--sysroot=${rootfs()}`,
            CGO_CFLAGS: `--sysroot=${rootfs()}`
        };
        await $`go build -o MiyooPod src/*.go`.env(env).cwd(wd);
        await $`install -Dm 0755 MiyooPod ${rootfs('usr/bin/MiyooPod')}`.cwd(wd);
        await $`mkdir -p ${rootfs('root/apps/MiyooPod')}`.cwd(wd);
        await $`mkdir -p ${rootfs('mnt/SDCARD/Media/Music')}`.cwd(wd);
        await $`mkdir -p ${rootfs('root/apps/MiyooPod/assets')}`.cwd(wd);
        await $`install -Dm 0644 App/MiyooPod/assets/icn.png ${rootfs('root/apps/MiyooPod/icon.png')}`.cwd(wd);
        await $`install -Dm 0644 App/MiyooPod/assets/ui_font.ttf ${rootfs('root/apps/MiyooPod/assets/ui_font.ttf')}`.cwd(wd);
        await Bun.write(rootfs('root/apps/MiyooPod/info.toml'), [
            `name="MiyooPod"`,
            `cmd="rg35xxp-guard MiyooPod"`,
            `icon="icon.png"`,
        ].join('\n'));
    }
}
