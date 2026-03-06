import {$} from "bun"
import type {Pack} from "../scripts/pkg.ts";
import {v} from "../scripts/helper.ts";

export const egui: Pack = {
    name: "rg35xxp-ui",
    version: "0.0.1",
    description: "UI",
    enable: true,
    dependencies: [],
    async sync({download}): Promise<void> {
    },
    async make({output, resource, rootfs, tar, build, env_autoconf}): Promise<void> {
        throw new Error('unimplemented')
    }
}

const init: Pack = {
    name: "rg35xxp-init",
    version: "0.0.1",
    description: "",
    enable: true,
    dependencies: [],
    async sync({download}): Promise<void> {
    },
    async make({output, resource, rootfs, tar, build, env_autoconf}): Promise<void> {
        await $`cargo build --release --target aarch64-unknown-linux-gnu --features file_logger`
            .env({
                ...process.env,
                PKG_CONFIG_SYSROOT_DIR: rootfs(),
                PKG_CONFIG_LIBDIR: `${rootfs('/usr/lib/pkgconfig')}:${rootfs('/usr/share/pkgconfig')}`,
                LDFLAGS: `--sysroot=${rootfs()} -L${rootfs('/usr/lib64')}`,
                CFLAGS: `--sysroot=${rootfs()}`,
                CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER: output('aarch64-linux-gnu/bin/aarch64-linux-gnu-gcc')
            })
            .cwd(resource("apps/rg35xxp-init"))
        const bin = resource("apps/rg35xxp-init/target/aarch64-unknown-linux-gnu/release/y-init");
        await $`cp ${bin} ${rootfs('/sbin/rg35xxp-init')}`
        await $`ln -sf rg35xxp-init ${rootfs('/sbin/init-ctrl')}`
        // 这里需要注意，由于没有设定好环境变量，导致动态链接库加载不到，所以直接软链接到 /init 会启动失败
        // 所以 /init 由脚本代替，设定好变量之后再启动 /sbin/rg35xxp-init
        const cfg = resource('apps/rg35xxp-init/init.toml')
        // await $`cp ${cfg} ${rootfs('init.toml')}`
    }
}
const launcher: Pack = {
    name: "rg35xxp-launcher",
    version: "0.0.1",
    description: "",
    enable: true,
    dependencies: [
        v("egl"),
    ],
    async sync({download}): Promise<void> {
    },
    async make({output, resource, rootfs, tar, build, env_autoconf}): Promise<void> {
        await $`cargo build --release --target aarch64-unknown-linux-gnu --features rg35xxp`
            .env({
                ...process.env,
                PKG_CONFIG_SYSROOT_DIR: rootfs(),
                PKG_CONFIG_LIBDIR: `${rootfs('/usr/lib/pkgconfig')}:${rootfs('/usr/share/pkgconfig')}`,
                LDFLAGS: `--sysroot=${rootfs()} -L${rootfs('/usr/lib64')}`,
                CFLAGS: `--sysroot=${rootfs()}`,
                CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER: output('aarch64-linux-gnu/bin/aarch64-linux-gnu-gcc')
            })
            .cwd(resource("apps/rg35xxp-iced"))
        const bin = resource("apps/rg35xxp-iced/target/aarch64-unknown-linux-gnu/release/rg35xxp-launcher")
        await $`cp ${bin} ${rootfs('/sbin/rg35xxp-launcher')}`
        await $`cp -r ${resource("apps/rg35xxp-iced/apps")} ${rootfs('/root/apps')}`
        await $`cp -r ${resource("apps/rg35xxp-iced/assets/material-design-icons")} ${rootfs('/usr/share/fonts/material-design-icons')}`
    }
}
const guard: Pack = {
    name: "rg35xxp-guard",
    version: "0.0.1",
    description: "",
    enable: true,
    dependencies: [],
    async sync({download}): Promise<void> {
    },
    async make({output, resource, rootfs, tar, build, env_autoconf}): Promise<void> {
        await $`cargo build --release --target aarch64-unknown-linux-gnu`
            .env({
                ...process.env,
                PKG_CONFIG_SYSROOT_DIR: rootfs(),
                PKG_CONFIG_LIBDIR: `${rootfs('/usr/lib/pkgconfig')}:${rootfs('/usr/share/pkgconfig')}`,
                LDFLAGS: `--sysroot=${rootfs()} -L${rootfs('/usr/lib64')}`,
                CFLAGS: `--sysroot=${rootfs()}`,
                CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER: output('aarch64-linux-gnu/bin/aarch64-linux-gnu-gcc')
            })
            .cwd(resource("apps/rg35xxp-guard"))
        const bin = resource("apps/rg35xxp-guard/target/aarch64-unknown-linux-gnu/release/rg35xxp-guard")
        await $`cp ${bin} ${rootfs('/sbin/rg35xxp-guard')}`
    }
}

export default {init, launcher, guard};