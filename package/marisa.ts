import type {Pack} from "../scripts/pkg.ts";
import {$} from "bun";

export const marisa: Pack = {
    name: "marisa",
    version: "0.2.6",
    description: "Static and space-efficient trie data structure library",
    enable: true,
    dependencies: [],
    async sync({download}): Promise<void> {
        return download({
            url: `https://github.com/s-yata/marisa-trie/archive/refs/tags/v0.2.6.tar.gz`,
            filename: `marisa-trie-0.2.6.tar.gz`
        })
    },
    async make({rootfs, tar, build, env_autoconf}): Promise<void> {
        const wd = build('marisa-trie-0.2.6');
        await $`[ ! -e marisa-trie-0.2.6 ] || rm -r marisa-trie-0.2.6`.cwd(build());
        await $`tar -xf ${tar(`marisa-trie-0.2.6.tar.gz`)}`.cwd(build());
        await $`autoreconf -i`.cwd(wd);
        await $`./configure --enable-shared --disable-static --host=aarch64-linux-gnu --prefix=/usr`.cwd(wd);
        await $`make -j16 && make install DESTDIR=${rootfs()}`.cwd(wd);
    }
}
