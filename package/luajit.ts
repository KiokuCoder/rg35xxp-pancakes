import type { Pack } from "../scripts/pkg.ts";
import { $ } from "bun";

const version = 'LuaJIT-2.1.ROLLING';
export const luajit: Pack = {
    name: "luajit",
    version: "2.1.ROLLING",
    description: "LuaJIT is a Just-In-Time Compiler for the Lua programming language.",
    enable: true,
    dependencies: [],
    async sync({ download }): Promise<void> {
        return download({
            url: `https://github.com/LuaJIT/LuaJIT/archive/refs/tags/v2.1.ROLLING.tar.gz`,
            filename: `${version}.tar.gz`
        })
    },
    async make({ rootfs, tar, build }): Promise<void> {
        await $`[ ! -e ${build(version)} ] || rm -r ${build(version)}`.cwd(build());
        await $`tar -xf ${tar(`${version}.tar.gz`)}`.cwd(build());
        const args = [
            "-j16",
            "PREFIX=/usr",
            "HOST_CC=gcc",
            "CROSS=aarch64-linux-gnu-",
            "TARGET_SYS=Linux",
            `DESTDIR=${rootfs()}`,
        ];
        await $`make ${args}`.cwd(build(version));
        await $`make ${args} install`.cwd(build(version));
    }
}
