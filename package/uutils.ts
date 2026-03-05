import type {Pack} from "../scripts/pkg.ts";
import {$} from "bun";

const version = 'coreutils-0.6.0'
export const coreutils: Pack = {
    name: "uutils-coreutils",
    version: "0.6.0",
    description: "",
    enable: true,
    dependencies: [],
    async sync({download}): Promise<void> {
        return download({
            url: `https://github.com/uutils/coreutils/archive/refs/tags/${version.split('-')[1]}.tar.gz`,
            filename: `${version}.tar.gz`
        })
    },
    async make({output, resource, rootfs, tar, build, env_autoconf}): Promise<void> {
        const wd = build(version);
        await $`[ ! -e ${build(version)} ] || rm -r ${build(version)}`.cwd(build());
        const tarball = tar(`${version}.tar.gz`);
        await $`tar -xf ${tarball}`.cwd(build());
        await $`cargo build --release --target aarch64-unknown-linux-gnu --features unix`
            .env({
                ...process.env,
                PKG_CONFIG_SYSROOT_DIR: rootfs(),
                PKG_CONFIG_LIBDIR: `${rootfs('/usr/lib/pkgconfig')}:${rootfs('/usr/share/pkgconfig')}`,
                LDFLAGS: `--sysroot=${rootfs()} -L${rootfs('/usr/lib64')}`,
                CFLAGS: `--sysroot=${rootfs()}`,
                CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER: output('aarch64-linux-gnu/bin/aarch64-linux-gnu-gcc')
            })
            .cwd(wd)
        await $`install -Dm 0755 target/aarch64-unknown-linux-gnu/release/coreutils ${rootfs("/usr/bin/coreutils")}`.cwd(wd);
        for (let tool of tools) {
            await $`ln -sf coreutils ${tool}`.cwd(rootfs('usr/bin'));
        }
    }
}

export const findutils: Pack = {
    name: "uutils-findutils",
    version: "0.6.0",
    description: "",
    enable: true,
    dependencies: [],
    async sync({download}): Promise<void> {
        return download({
            url: `https://github.com/uutils/findutils/archive/refs/tags/0.8.0.tar.gz`,
            filename: `findutils-0.8.0.tar.gz`
        })
    },
    async make({output, resource, rootfs, tar, build, env_autoconf}): Promise<void> {
        const wd = build('findutils-0.8.0');
        await $`[ ! -e ${build(version)} ] || rm -r ${build(version)}`.cwd(build());
        const tarball = tar(`findutils-0.8.0.tar.gz`);
        await $`tar -xf ${tarball}`.cwd(build());
        await $`cargo build --release --target aarch64-unknown-linux-gnu`
            .env({
                ...process.env,
                PKG_CONFIG_SYSROOT_DIR: rootfs(),
                PKG_CONFIG_LIBDIR: `${rootfs('/usr/lib/pkgconfig')}:${rootfs('/usr/share/pkgconfig')}`,
                LDFLAGS: `--sysroot=${rootfs()} -L${rootfs('/usr/lib64')}`,
                CFLAGS: `--sysroot=${rootfs()}`,
                CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER: output('aarch64-linux-gnu/bin/aarch64-linux-gnu-gcc')
            })
            .cwd(wd)
        await $`install -Dm 0755 target/aarch64-unknown-linux-gnu/release/xargs ${rootfs("/usr/bin/xargs")}`.cwd(wd);
        await $`install -Dm 0755 target/aarch64-unknown-linux-gnu/release/find ${rootfs("/usr/bin/find")}`.cwd(wd);
    }
}


const tools = [
    "arch", "b2sum", "base32", "base64", "basename", "basenc",
    "cat", "chgrp", "chmod", "chown", "chroot", "cksum", "comm", "cp", "csplit", "cut",
    "date", "dd", "df", "dir", "dircolors", "dirname", "du",
    "echo", "env", "expand", "expr", "factor", "false", "fmt", "fold",
    "groups", "head", "hostid", "hostname",
    "id", "install", "join", "kill", "link", "ln", "logname", "ls", "md5sum", "mkdir", "mkfifo", "mknod",
    "mktemp", "more", "mv", "nice", "nl", "nohup", "nproc", "numfmt", "od", "paste", "pathchk", "pinky",
    "pr", "printenv", "printf", "ptx", "pwd", "readlink", "realpath", "rm", "rmdir", "seq", "sha1sum", "sha224sum",
    "sha256sum", "sha384sum", "sha512sum", "shred", "shuf", "sleep", "sort", "split", "stat", "stdbuf", "stty",
    "sum", "sync", "tac", "tail", "tee", "test", "timeout", "touch", "tr", "true", "truncate", "tsort", "tty",
    "uname", "unexpand", "uniq", "unlink", "uptime", "users", "vdir", "wc", "who", "whoami",
    "yes", "[",]
export default {coreutils, findutils}