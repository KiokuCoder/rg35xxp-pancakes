import {$} from "bun"
import type {Pack} from "../scripts/pkg.ts";

export const openssl: Pack = {
    name: "openssl",
    version: "3.3.2",
    description: "",
    enable: false,
    dependencies: [],
    async sync({download}): Promise<void> {
        return download({
            url: "https://github.com/openssl/openssl/releases/download/openssl-3.3.2/openssl-3.3.2.tar.gz",
            filename: "openssl-3.3.2.tar.gz",
            md5: "015fca2692596560b6fe8a2d8fecd84b"
        })
    },
    async make({rootfs, tar, build, env_autoconf}): Promise<void> {
        await $`[ ! -e ${build('openssl-3.3.2')} ] || rm -r ${build('openssl-3.3.2')}`.cwd(build());
        await $`tar -xf ${tar('openssl-3.3.2.tar.gz')}`.cwd(build());
        await $`./config linux-aarch64 --openssldir=/etc/ssl --prefix=/usr --cross-compile-prefix=aarch64-linux-gnu-`.cwd(build('openssl-3.3.2'));
        await $`make -j16`.env(env_autoconf()).cwd(build('openssl-3.3.2'));
        await $`make DESTDIR=${rootfs()} install_sw`.env(env_autoconf()).cwd(build('openssl-3.3.2'));
    }
}
