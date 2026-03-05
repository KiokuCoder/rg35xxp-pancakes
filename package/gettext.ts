import {$} from "bun"
import type {Pack} from "../scripts/pkg.ts";

const version = 'gettext-0.21';
export const gettext: Pack = {
    name: "gettext",
    version: "0.21",
    description: "GNU internationalization (i18n) and localization (l10n) library",
    enable: false,
    dependencies: [],
    async sync({download}): Promise<void> {
        return download({
            url: `https://ftp.gnu.org/pub/gnu/gettext/${version}.tar.xz`,
            filename: `${version}.tar.xz`
        })
    },
    async make({resource, rootfs, tar, build, env_autoconf}): Promise<void> {
        await $`[ ! -e ${build(version)} ] || rm -r ${build(version)}`.cwd(build());
        await $`tar -xf ${tar(`${version}.tar.xz`)}`.cwd(build());

        const env = {
            ...process.env,
            PKG_CONFIG_PATH: "",
            PKG_CONFIG_LIBDIR: `${rootfs()}/usr/lib/pkgconfig:${rootfs()}/usr/share/pkgconfig`,
            PKG_CONFIG_SYSROOT_DIR: `${rootfs()}`
        }

        const args = [
            '--host=aarch64-linux-gnu',
            '--prefix=/usr',
            '--disable-java',
            '--disable-csharp',
            '--disable-c++',
            '--disable-libxml2',
            '--with-included-glib',
            '--with-included-libcroco',
            '--with-included-libunistring',
            '--with-included-gettext',
            '--with-included-libexpat',
            '--disable-openmp',

        ];
        await $`./configure ${args}`.env(env).cwd(build(version));
        await $`make -j16`.cwd(build(version));
        await $`make install DESTDIR=${rootfs()}`.cwd(build(version));
    }
}
