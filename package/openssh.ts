import {$} from "bun"
import type {Pack} from "../scripts/pkg.ts";
import {configureMakeMakeInstall} from "../scripts/helper.ts";


const version = 'openssh-9.2p1';
export const openssh: Pack = {
    name: "openssh",
    version: version.split('-')[1],
    description: "scp is a command-line utility for securely copying files between hosts.",
    enable: false,
    dependencies: [
        {
            name: "zlib",
            version: "*",
            required: true
        },
        {
            name: "openssl",
            version: "*",
            required: true
        }
    ],
    async sync({download}): Promise<void> {
        return download({
            url: `https://cdn.openbsd.org/pub/OpenBSD/OpenSSH/portable/${version}.tar.gz`,
            filename: `${version}.tar.gz`
        })
    },
    async make({rootfs, tar, build, env_autoconf}): Promise<void> {
        await $`[ ! -e ${build(version)} ] || rm -r ${build(version)}`.cwd(build());
        await $`tar -xf ${tar(`${version}.tar.gz`)}`.cwd(build());

        const wd = build(version);
        const env = env_autoconf();
        const args = [
            '--prefix=/usr',
            '--host=aarch64-linux-gnu',
            '--disable-strip',
        ]
        await $`./configure CFLAGS=-w ${args}`.env(env).cwd(wd);
        await $`make -j16`.env(env).cwd(wd);
        await $`make DESTDIR=${rootfs()} install-files install-sysconf`.env(env).cwd(wd);

    }
}
