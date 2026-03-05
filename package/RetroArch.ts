import type { Pack } from "../scripts/pkg.ts";
import { $ } from "bun";

export const RetroArch: Pack = {
    name: "RetroArch",
    version: "1.19.1",
    description: "",
    enable: false,
    dependencies: [
        {
            name: "eudev",
            version: "> 1.0.0",
            required: true
        },
        {
            name: "alsa",
            version: "*",
            required: true
        },
        {
            name: "ffmpeg",
            version: "*",
            required: false
        },
        {
            name: "mpv",
            version: "*",
            required: false
        }
    ],
    async sync({ download }): Promise<void> {
        return download({
            filename: "RetroArch-1.19.1.tar.gz",
            url: "https://github.com/libretro/RetroArch/archive/refs/tags/v1.19.1.tar.gz"

        });
    },
    async make({ resource, rootfs, tar, build, env_autoconf }): Promise<void> {
        const src = build('RetroArch-1.19.1');
        await $`[ ! -e ${src} ] || rm -r ${src}`;
        await $`tar -xf ${tar(`RetroArch-1.19.1.tar.gz`)}`.cwd(build());
        // don't add --enable-dynamic_egl otherwise will cause Segmentation fault, I do not know why
        const args0 = [

            // 安装目录
            '--prefix=/usr',
            '--host=aarch64-linux-gnu',
            '--enable-mali_fbdev',
            '--enable-egl',
            '--enable-alsa',
            // '--enable-tinyalsa',
            // '--enable-dynamic_egl', // 这个开起来的话也会错误
            '--disable-wayland',
            '--disable-systemd',
            '--disable-sdl',
            '--disable-sdl2',
            '--enable-opengles',
            '--enable-ffmpeg',

            // UI config
            '--enable-wifi',
            '--enable-materialui',
            '--enable-xmb',
            '--enable-ozone',
            '--disable-ssa' // libass 貌似存在问题，如果开启这个插件，就会编译错误
        ]
        await $`./configure ${args0}`.env(env_autoconf()).cwd(src);
        await $`make -j16`.env(env_autoconf()).cwd(src);
        await $`make install DESTDIR=${rootfs()}`.env(env_autoconf()).cwd(src);
        await $`cat ${resource('retroarch.cfg')} >> ${rootfs('etc/retroarch.cfg')}`
    },
}
export const RetroArchAssets: Pack = {
    name: "RetroArchAssets",
    version: "1.19.0",
    description: "",
    enable: false,
    dependencies: [],
    async sync({ download }): Promise<void> {
        await download({
            filename: "retroarch-assets-1.19.0.tar.gz",
            url: "https://github.com/libretro/retroarch-assets/archive/refs/tags/v1.19.0.tar.gz"
        })
        await download({
            filename: "retroarch-joypad-autoconfig-1.19.0.tar.gz",
            url: "https://github.com/libretro/retroarch-joypad-autoconfig/archive/refs/tags/v1.19.0.tar.gz"
        })
    },
    async make({ resource, rootfs, tar, build, env_autoconf }): Promise<void> {
        const src = build('retroarch-assets-1.19.0');
        await $`[ ! -e ${src} ] || rm -r ${src}`;
        await $`tar -xf ${tar('retroarch-assets-1.19.0.tar.gz')}`.cwd(build())
        await $`make install DESTDIR=${rootfs()}`.cwd(src)

        // 只保留 GLUI 主题和必要文件，删除其他大体积主题目录 (~530M -> ~10M)
        const assetsDir = rootfs('usr/share/libretro/assets');
        const removeDirs = [
            'Automatic', 'FlatUX', 'Systematic', 'xmb', 'ozone',
            'wallpapers', 'nxrgui', 'switch', 'pkg', 'branding', 'ctr', 'devtools'
        ];
        for (const dir of removeDirs) {
            await $`rm -rf ${assetsDir}/${dir}`;
        }

        await $`[ ! -e ${build('retroarch-joypad-autoconfig-1.19.0')} ] || rm -r ${build('retroarch-joypad-autoconfig-1.19.0')}`;
        await $`tar -xf ${tar('retroarch-joypad-autoconfig-1.19.0.tar.gz')}`.cwd(build())
        await $`make install DESTDIR=${rootfs()}`.cwd(build('retroarch-joypad-autoconfig-1.19.0'))
        await $`install -Dm 0644 ${resource('Deeplay-keys.cfg')} ${rootfs('usr/share/libretro/autoconfig/udev/Deeplay-keys.cfg')}`
        await $`install -Dm 0644 ${resource('Deeplay-keys.cfg')} ${rootfs('root/.config/retroarch/autoconfig/Deeplay-keys.cfg')}`
        await $`install -Dm 0644 ${resource('ANBERNIC-keys.cfg')} ${rootfs('usr/share/libretro/autoconfig/udev/ANBERNIC-keys.cfg')}`
        await $`install -Dm 0644 ${resource('ANBERNIC-keys.cfg')} ${rootfs('root/.config/retroarch/autoconfig/ANBERNIC-keys.cfg')}`
    },
}
