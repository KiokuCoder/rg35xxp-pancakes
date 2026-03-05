import {$} from "bun"
import type {Pack} from "../scripts/pkg.ts";

export const qt5: Pack = {
    name: "qt5",
    version: "5.15.16",
    description: "Qt is a cross-platform application and UI framework for developers using C++ or QML, a CSS & JavaScript like language.",
    enable: false,
    dependencies: [
        {
            name: "zlib",
            version: "*",
            required: true
        },
        {
            name: "egl",
            version: "*",
            required: true
        },
        {
            name: "openssl",
            version: "*",
            required: false
        },
        {
            name: "freetype2",
            version: "*",
            required: false
        },
        {
            name: "harfbuzz",
            version: "*",
            required: true
        },
        {
            name: "libpng",
            version: "*",
            required: false
        },
        {
            name: "libjpeg",
            version: "*",
            required: false
        },
    ],
    async sync({download}): Promise<void> {
        return download({
            url: `https://download.qt.io/archive/qt/5.15/5.15.16/submodules/qtbase-everywhere-opensource-src-5.15.16.tar.xz`,
            filename: `qtbase-everywhere-opensource-src-5.15.16.tar.xz`
        })
    },
    async make({rootfs, tar, build, env_autoconf}): Promise<void> {
        const version = 'qtbase-everywhere-src-5.15.16';
        const wd = build(version);
        await $`[ ! -e ${version} ] || rm -r ${version}`.cwd(build());
        await $`tar -xf ${tar(`qtbase-everywhere-opensource-src-5.15.16.tar.xz`)}`.cwd(build());
        const env = {
            ...process.env,
            PKG_CONFIG_PATH: "",
            PKG_CONFIG_LIBDIR: `${rootfs()}/usr/lib/pkgconfig:${rootfs()}/usr/share/pkgconfig`,
            PKG_CONFIG_SYSROOT_DIR: `${rootfs()}`,
        }
        const args = [
            '-verbose',
            '-recheck-all',
            '-opensource',
            '-confirm-license',
            '-release',
            '-sysroot', rootfs(),
            '-xplatform', 'linux-aarch64-gnu-g++',
            '-make', 'libs',
            '-egl',
        ];
        await $`./configure ${args}`.env(env).cwd(wd);
        await $`gmake -j16`.cwd(wd);
        await $`gmake install`.cwd(wd);
    }
}