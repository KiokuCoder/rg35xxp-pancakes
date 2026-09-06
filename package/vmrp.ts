import type {Pack} from "../scripts/pkg.ts";
import {$} from "bun";
import {v} from "../scripts/helper.ts";

const home = 'root/apps/vmrp';

export const vmrp: Pack = {
    name: "vmrp",
    version: "1.0.0",
    description: "vmrp is a mrp(mythroad) simulator based on the unicorn engine.",
    enable: false,
    dependencies: [
        v("unicorn"),
        v("zlib"),
        {name: "sdl2", version: "*", required: true},
        // sound.c 用它们解 MIDI / MP3
        v("libadlmidi"),
        v("libmpg123"),
    ],
    async sync({download}): Promise<void> {
    },
    async make({rootfs, resource, env_autoconf}): Promise<void> {
        const wd = resource('apps/vmrp');
        const env = env_autoconf();

        await $`make clean`.env(env).cwd(wd);
        await $`make -j16 VMRP_HOME=${`/${home}`}`.env(env).cwd(wd);
        await $`install -Dm 0755 build/vmrp ${rootfs('usr/bin/vmrp')}`.cwd(wd);

        // 运行时依赖 cfunction.ext 和 mythroad 目录，且都是相对当前目录查找的。
        // 源码里让 vmrp 找不到 cfunction.ext 时自动 chdir 到 VMRP_HOME，所以不需要 wrapper 脚本
        await $`mkdir -p ${rootfs(home)}`.cwd(wd);
        await $`cp -r fs/cfunction.ext fs/mythroad ${rootfs(home)}`.cwd(wd);

        // 客户机画面是 240x320 的竖屏，掌机屏幕 640x480，不转的话只占左上角一小块。
        // 顺时针转 90 度之后是 320x240，正好整数放大 2 倍铺满屏幕，方向键跟着一起转
        await Bun.write(rootfs(home, 'info.toml'), [
            `name="vmrp"`,
            `cmd="rg35xxp-guard vmrp --rotate 90"`,
            `icon="icon.png"`,
        ].join('\n'));
    }
}
