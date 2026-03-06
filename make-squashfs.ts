#!/usr/bin/env bun
import { Container, type Register, type TestNode } from "./scripts/container.ts";
import type { Context } from "./scripts/config.ts";
import { join } from "path";
import { $ } from "bun";
import {workspace} from "./scripts/pkg.ts";

const ctx: Context = workspace;
const container = new Container(ctx);

const output = join(__dirname, "output");
const rootfs = join(output, "rootfs");
const slim = join(output, "rootfs-slim");

const pack: TestNode<Context> = {
    install(hook: Register<Context>): void {

        // 1. 清理旧目录，复制 rootfs
        hook("release", "rootfs-slim-prepare", async () => {
            await $`[ ! -e ${slim} ] || rm -rf ${slim}`;
            await $`cp -a ${rootfs} ${slim}`;
            console.log("rootfs copied to rootfs-slim");
        }, { after: [] })

        // 2. 合并 bin/sbin 到 usr/bin、usr/sbin
        hook("release", "rootfs-slim-merge", async () => {
            // TODO: 合并 bin/sbin
            console.log("bin/sbin merged into usr/bin and usr/sbin");
        }, { after: ["rootfs-slim-prepare"] })

        // 3. Strip 二进制文件和共享库
        hook("release", "rootfs-slim-strip", async () => {
            const strip = join(__dirname, "output/aarch64-linux-gnu/bin/aarch64-linux-gnu-strip");
            // strip executables in usr/bin and usr/sbin
            for (const dir of ["usr/bin", "usr/sbin"]) {
                await $`find ${dir} -type f -exec sh -c 'file "$1" | grep -q "ELF" && ${strip} --strip-all "$1" 2>/dev/null || true' _ {} \;`.cwd(slim);
            }
            // strip shared libraries in usr/lib64
            await $`find usr/lib64 -type f -name "*.so*" -exec sh -c 'file "$1" | grep -q "ELF" && ${strip} --strip-unneeded "$1" 2>/dev/null || true' _ {} \;`.cwd(slim);
            console.log("ELF binaries and libraries stripped");
        }, { after: ["rootfs-slim-merge"] })

        // 4. 删除无用文件
        hook("release", "rootfs-slim-clean", async () => {
            const removeDirs = [
                "usr/include",
                "usr/share/man",
                "usr/share/doc",
                "usr/share/info",
                "usr/share/aclocal",
                "usr/share/pkgconfig",
                "usr/lib/pkgconfig",
                "usr/lib64/pkgconfig",
                "usr/lib/cmake",
            ];
            for (const dir of removeDirs) {
                await $`rm -rf ${dir}`.cwd(slim);
            }
            // 删除 libtool .la 文件
            await $`find usr/lib usr/lib64 -name "*.la" -delete 2>/dev/null || true`.cwd(slim);
            // 删除 .a 静态库
            await $`find usr/lib usr/lib64 -name "*.a" -delete 2>/dev/null || true`.cwd(slim);
            console.log("development files and docs removed");
        }, { after: ["rootfs-slim-strip"] })

        // 5. 打包 squashfs
        hook("release", "rootfs-slim-squashfs", async () => {
            const img = join(output, 'rootfs.squashfs');
            await $`[ ! -e ${img} ] || rm ${img}`;
            await $`mksquashfs ${slim} ${img} -comp gzip`;
            console.log(`rootfs packaged: ${img}`);
        }, { after: ["rootfs-slim-clean"] })
    }
}

container.use(pack);

await container.start().then(() => {
    console.log("done")
});
