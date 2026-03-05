import {$} from "bun";
import {join} from "path";
import type {Dependency} from "./pkg.ts";

export function v(name: string): Dependency {
    return {
        name: name, required: true, version: "*"
    }
}

const bin = join(__dirname, "../output/aarch64-linux-gnu/bin/")

export async function configureMakeMakeInstall(wd: string, rootfs: string, env: Record<string, string>, args: string[]) {
    const args0 = [

        // 安装目录
        '--prefix=/usr',
        '--host=aarch64-linux-gnu',
        '--disable-static',
        '--enable-shared',
        `--with-sysroot=${rootfs}`
    ]
    await $`./configure CFLAGS=-w ${args0} ${args}`.env(env).cwd(wd);
    await $`make -j16`.env(env).cwd(wd);
    await $`make install DESTDIR=${rootfs}`.env(env).cwd(wd);
}

export async function mesonBuild(wd: string, rootfs: string, args: string[]) {
    await Bun.write(join(wd, "cross_file.txt"), mesonCrossFile(rootfs));
    console.log(`cd ${wd}`)
    console.log(`meson setup __build ${args} --prefix=/usr --cross-file cross_file.txt`)
    await $`meson setup __build ${args} --prefix=/usr --cross-file cross_file.txt`.cwd(wd);
    await $`meson compile -C __build -j16`.cwd(wd);
    await $`DESTDIR=${rootfs} meson install -C __build`.cwd(wd);
}

export function mesonCrossFile(root: string): string {
    return [
        `[binaries]`,
        `c = 'aarch64-linux-gnu-gcc'`,
        `cpp = 'aarch64-linux-gnu-g++'`,
        `ar = 'aarch64-linux-gnu-ar'`,
        `strip = 'aarch64-linux-gnu-strip'`,
        `pkgconfig = 'aarch64-linux-gnu-pkg-config'`,
        ``,
        `[host_machine]`,
        `system = 'linux'`,
        `cpu_family = 'aarch64'`,
        `cpu = 'aarch64'`,
        `endian = 'little'`,
        ``,
        `[properties]`,
        `# 指定 sysroot 路径`,
        `sys_root = '${root}'`,
        `# 设置用于 pkg-config 的 sysroot`,
        `pkg_config_libdir = '${root}/usr/lib/pkgconfig:${root}/usr/share/pkgconfig'`,
    ].join('\n');
}


export async function cmakeBuild(wd: string, rootfs: string, env: Record<string, string>, args: string[]) {
    await $`mkdir __build`.cwd(wd);
    await $`cmake -G"Unix Makefiles" -DENABLE_STATIC=OFF ${cmakeArgs(rootfs)} ${args} ..`.cwd(join(wd, "__build"));
    await $`make -j16`.cwd(join(wd, '__build'));
    await $`make install DESTDIR=${rootfs}`.cwd(join(wd, '__build'));
}

export function cmakeArgs(root: string) {
    const env: Record<string, string> = {
        CMAKE_SYSTEM_NAME: "Linux",
        CMAKE_SYSTEM_PROCESSOR: "aarch64",
        CMAKE_FIND_ROOT_PATH: root,
        CMAKE_INSTALL_PREFIX: '/usr',
        CMAKE_FIND_ROOT_PATH_MODE_LIBRARY: "ONLY",
        CMAKE_FIND_ROOT_PATH_MODE_INCLUDE: "ONLY",
        CMAKE_C_COMPILER: `${bin}aarch64-linux-gnu-gcc`,
        CMAKE_CXX_COMPILER: `${bin}aarch64-linux-gnu-g++`,
        CMAKE_AR: `${bin}aarch64-linux-gnu-ar`,
        CMAKE_AS: `${bin}aarch64-linux-gnu-as`,
        CMAKE_RANLIB: `${bin}aarch64-linux-gnu-ranlib`,
        CMAKE_OBJCOPY: `${bin}aarch64-linux-gnu-objcopy`,
        CMAKE_OBJDUMP: `${bin}aarch64-linux-gnu-objdump`,
        CMAKE_LINKER: `${bin}aarch64-linux-gnu-ld`,
        CMAKE_NM: `${bin}aarch64-linux-gnu-nm`,
        CMAKE_STRIP: `${bin}aarch64-linux-gnu-strip`,
        PKG_CONFIG_EXECUTABLE: `${bin}aarch64-linux-gnu-pkg-config`,
    }

    const args: string[] = [];
    for (let k in env) {
        args.push(`-D${k}=${env[k]}`);
    }
    return args;
}
