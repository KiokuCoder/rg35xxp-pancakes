import {$} from "bun"
import type {Pack} from "../scripts/pkg.ts";
import {configureMakeMakeInstall} from "../scripts/helper.ts";


const version = 'gnutls-3.7.8';
export const gnutls: Pack = {
    name: "gnutls",
    version: "3.7.8",
    description: "GnuTLS is a secure communications library implementing the SSL, TLS and DTLS protocols and technologies around them.",
    enable: false,
    dependencies: [
        {
            name: "zlib",
            version: "> 1.0.0",
            required: true,
        },
        {
            name: "nettle",
            version: "*",
            required: true,
        },
    ],
    async sync({download}): Promise<void> {
        return download({
            url: `https://www.gnupg.org/ftp/gcrypt/gnutls/v3.7/${version}.tar.xz`,
            filename: `${version}.tar.xz`
        })
    },
    async make({rootfs, tar, build, env_autoconf}): Promise<void> {
        await $`[ ! -e ${build(version)} ] || rm -r ${build(version)}`.cwd(build());
        await $`tar -xf ${tar(`${version}.tar.xz`)}`.cwd(build());

        const configureArgs = [
            `--with-included-libtasn1`, // If libtasn1 is a dependency
            '--with-included-unistring',
            '--without-p11-kit',
            `--disable-static`, // Often preferred for size and flexibility
            `--enable-shared`,
            `--disable-tools`, // If you don't need the command-line tools
            `--disable-tests`, // To skip building tests
        ];


        await configureMakeMakeInstall(build(version), rootfs(), env_autoconf(), configureArgs);
    }
}
