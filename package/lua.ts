import type {Pack} from "../scripts/pkg.ts";
import {$} from "bun";

export const lua: Pack = {
    name: "lua",
    version: "5.4.6",
    description: "",
    enable: false,
    dependencies: [],
    async sync({download}): Promise<void> {
        return download({
            url: `https://www.lua.org/ftp/lua-5.4.6.tar.gz`,
            filename: `lua-5.4.6.tar.gz`
        });
    },
    async make({resource, rootfs, tar, build, env_autoconf}): Promise<void> {
        const src = build("lua-5.4.6")
        await $`[ ! -e ${src} ] || rm -r ${src}`.cwd(build());
        await $`tar -xf ${tar(`lua-5.4.6.tar.gz`)}`.cwd(build());
        await $`make -j16`.env(env_autoconf()).cwd(src)
        await $`make install INSTALL_TOP=${rootfs()}/usr`.cwd(src);
    }
}
