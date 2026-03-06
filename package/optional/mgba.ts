import {$} from "bun"
import {cmakeArgs, v} from "../../scripts/helper.ts";
import type {Pack} from "../../scripts/pkg.ts";

const version = 'mgba-0.10.5';
export const mgba0: Pack = {
    name: "mgba",
    version: "0.10.5",
    description: "mGBA is a new emulator for running Game Boy Advance games.",
    enable: false,
    dependencies: [v("pixman"), v("libpng"), v("sdl2")],
    async sync({download}): Promise<void> {
        return download({
            url: `https://github.com/mgba-emu/mgba/archive/refs/tags/${version.split('-')[1]}.tar.gz`,
            filename: `${version}.tar.gz`
        })
    },
    async make({resource, rootfs, tar, build, env_autoconf}): Promise<void> {
        const env = {
            ...process.env,
            PKG_CONFIG_PATH: "",
            PKG_CONFIG_LIBDIR: `${rootfs()}/usr/lib/pkgconfig:${rootfs()}/rootfs/usr/share/pkgconfig`,
            PKG_CONFIG_SYSROOT_DIR: rootfs(),
        }
        await $`[ ! -e ${build(version)} ] || rm -r ${build(version)}`.cwd(build());
        await $`tar -xf ${tar(`${version}.tar.gz`)}`.cwd(build());
        // await $`[ ! -e build ] || rm -r build`.cwd(build(version));
        await $`mkdir build`.cwd(build(version));
        await $`cmake -DENABLE_SCRIPTING=OFF -DBUILD_QT=OFF -DBUILD_SDL=ON -DBUILD_GLES2=ON ${cmakeArgs(rootfs())} ..`.env(env).cwd(build(version, 'build'));
        await $`make -j16`.cwd(build(version, 'build'));
        await $`make install DESTDIR=${rootfs()}`.cwd(build(version, 'build'));
    }
}
// 0.10.5 版本还是存在 sdl 输入问题
export const mgba: Pack = {
    name: "mgba",
    version: "0.10.5",
    description: "mGBA is a new emulator for running Game Boy Advance games.",
    enable: false,
    dependencies: [v("pixman"), v("libpng"), v("sdl2")],
    async sync({download}): Promise<void> {
    },
    async make({resource, rootfs, tar, build, env_autoconf}): Promise<void> {
        const env = {
            ...process.env,
            PKG_CONFIG_PATH: "",
            PKG_CONFIG_LIBDIR: `${rootfs()}/usr/lib/pkgconfig:${rootfs()}/rootfs/usr/share/pkgconfig`,
            PKG_CONFIG_SYSROOT_DIR: rootfs(),
        }
        const b = build('mgba/build');
        const wd = build('mgba');
        await $`[ ! -e ${wd} ] || rm -r ${wd}`.cwd(build());
        await $`git clone https://github.com/mgba-emu/mgba.git mgba`.cwd(build());
        await $`git checkout ea50b5e876259ca0423af7b4a3335ff9fc10c388`.cwd(wd);
        await $`mkdir build`.cwd(wd);
        await $`cmake -DENABLE_SCRIPTING=OFF -DBUILD_QT=OFF -DBUILD_SDL=ON -DBUILD_GLES2=ON ${cmakeArgs(rootfs())} ..`.env(env).cwd(b);
        await $`make -j16`.cwd(b);
        await $`make install DESTDIR=${rootfs()}`.cwd(b);
    }
}
