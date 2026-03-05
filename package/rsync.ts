import {$} from "bun"
import type {Pack} from "../scripts/pkg.ts";
import {configureMakeMakeInstall, v} from "../scripts/helper.ts";


const version = 'rsync-3.2.7';
export const rsync: Pack = {
    name: "rsync",
    version: "3.2.7",
    description: "rsync is a utility for efficiently transferring and synchronizing files.",
    enable: true,
    dependencies: [
        v("zlib"),
        v("openssl")
    ],
    async sync({download}): Promise<void> {
        return download({
            url: `https://download.samba.org/pub/rsync/src/${version}.tar.gz`,
            filename: `${version}.tar.gz`
        })
    },
    async make({rootfs, tar, build, env_autoconf}): Promise<void> {
        await $`[ ! -e ${build(version)} ] || rm -r ${build(version)}`.cwd(build());
        await $`tar -xf ${tar(`${version}.tar.gz`)}`.cwd(build());
        await configureMakeMakeInstall(build(version), rootfs(), env_autoconf({}), [
            '--disable-xxhash',
            '--disable-zstd',
            '--disable-lz4',
        ])
    }
}
