import type {Register, TestNode} from "./container.ts";
import {$} from "bun"
import type {Context} from "./config.ts";
import {join} from "path";

/**
 * 1. 编译 binutils
 * 2. 编译 gcc0，sysroot 为临时 sysroot 目录 build('root')
 * 3. 安装 linux 头文件到临时 sysroot 目录 build('root')
 * 4. 编译 glibc0，安装到临时 sysroot 目录 build('root')
 * 5. 编译最终 gcc，安装到 prefix 目录
 * 6. 编译 glibc，安装到 sysroot 目录 rootfs('usr')
 */
const v = {
    binutils: "binutils-2.34",
    gcc: "gcc-13.3.0",
    gmp: "gmp-6.3.0",
    mpc: "mpc-1.3.1",
    mpfr: "mpfr-4.2.1",
    glibc: "glibc-2.38",
    linux: "linux-4.9.170",
    pkgConfig: "pkg-config-0.29.2"
} as const;

export const toolchain: TestNode<Context> = {
    install(hook: Register<Context>): void {
        hook("sync", "download", async (ctx, {download}) => {
            await download({
                url: `https://mirrors.tuna.tsinghua.edu.cn/gnu/binutils/${v.binutils}.tar.xz`,
                filename: `${v.binutils}.tar.xz`,
            })
            await download({
                url: `https://mirrors.tuna.tsinghua.edu.cn/gnu/gcc/${v.gcc}/${v.gcc}.tar.xz`,
                filename: `${v.gcc}.tar.xz`,
            })
            await download({
                url: `https://mirrors.tuna.tsinghua.edu.cn/gnu/gmp/${v.gmp}.tar.xz`,
                filename: `${v.gmp}.tar.xz`,
            })
            await download({
                url: `https://mirrors.tuna.tsinghua.edu.cn/gnu/mpc/${v.mpc}.tar.gz`,
                filename: `${v.mpc}.tar.gz`,
            })
            await download({
                url: `https://mirrors.tuna.tsinghua.edu.cn/gnu/mpfr/${v.mpfr}.tar.xz`,
                filename: `${v.mpfr}.tar.xz`,
            })
            await download({
                url: `https://mirrors.tuna.tsinghua.edu.cn/gnu/glibc/${v.glibc}.tar.xz`,
                filename: `${v.glibc}.tar.xz`,
            })
            const m = v.linux.match(/linux-(\d+)\./)?.[1] || "4";
            await download({
                url: `https://mirrors.tuna.tsinghua.edu.cn/kernel/v${m}.x/${v.linux}.tar.xz`,
                filename: `${v.linux}.tar.xz`,
            })
            await download({
                url: `https://pkgconfig.freedesktop.org/releases/${v.pkgConfig}.tar.gz`,
                filename: `${v.pkgConfig}.tar.gz`,
            })
        }, {after: []})

        hook("sync", "clear", async (ctx, {rootfs,build}) => {
            for (let p of ["gcc", "gcc0", "glibc0", "glibc", "binutils"]) {
                await $`[ -e ${build(p)} ] && rm -r ${build(p)} || true`
            }

            await $`[ ! -e ${v.gmp}  ] || rm -r ${v.gmp} `.cwd(build())
            await $`[ ! -e ${v.mpc}  ] || rm -r ${v.mpc} `.cwd(build())
            await $`[ ! -e ${v.mpfr} ] || rm -r ${v.mpfr}`.cwd(build())
            await $`[ ! -e ${rootfs()} ] || rm -r ${rootfs()}`.cwd(build())

        }, {after: []});

        hook("sync", "unpack", async (ctx, {build, tar, output}) => {
            await $`[ -e ${output('aarch64-linux-gnu')} ] && rm -r ${output('aarch64-linux-gnu')} || true`
            const files = [`${v.glibc}.tar.xz`, `${v.linux}.tar.xz`, `${v.gcc}.tar.xz`, `${v.gmp}.tar.xz`,
                `${v.mpc}.tar.gz`, `${v.mpfr}.tar.xz`, `${v.binutils}.tar.xz`]

            for (let file of files) {
                console.log(`extract file: ${file}`)
                await $`tar -xf ${tar(file)}`.cwd(build());
            }

            await $`[ ! -e ${v.gcc}/gmp  ] || rm -r ${v.gcc}/gmp`.cwd(build())
            await $`[ ! -e ${v.gcc}/mpc  ] || rm -r ${v.gcc}/mpc`.cwd(build())
            await $`[ ! -e ${v.gcc}/mpfr ] || rm -r ${v.gcc}/mpfr`.cwd(build())
            await $`mv ${v.gmp}  ${v.gcc}/gmp`.cwd(build())
            await $`mv ${v.mpc}  ${v.gcc}/mpc`.cwd(build())
            await $`mv ${v.mpfr} ${v.gcc}/mpfr`.cwd(build())
        }, {after: ['download', 'clear']});


        hook("compile", "binutils", async (ctx, {tar, output, build, rootfs}) => {
            await $`mkdir -p ${build("binutils")}`;
            await $`mkdir -p ${rootfs()}`;

            const args = [
                `--prefix=${output('aarch64-linux-gnu')}`,
                '--target=aarch64-linux-gnu',
                '--disable-multilib',
                `--with-sysroot=${rootfs()}`
            ];
            await $`${build(`${v.binutils}/configure`)} ${args}`.cwd(build("binutils"));
            await $`make -j16`.cwd(build("binutils"));
            await $`make install`.cwd(build("binutils"));

        }, {after: []})

        hook("compile", "gcc0", async (ctx, {tar, build, output}) => {
            await $`mkdir -p ${build("gcc0")}`;

            const args = [
                `--prefix=${output('aarch64-linux-gnu')}`,
                '--target=aarch64-linux-gnu',
                '--enable-languages=c,c++',
                '--without-headers',
                '--disable-werror',
                '--disable-nls',
                '--disable-shared',
                '--disable-multilib',
                '--disable-threads',
                '--disable-libatomic',
                '--disable-libgomp',
                '--disable-libquadmath',
                '--disable-libssp',
                '--disable-libvtv',
                '--disable-libstdcxx',
                '--disable-multilib',
            ];
            console.log(`build gcc0: ${build(`${v.gcc}/configure`)} ${args}`)
            await $`${build(`${v.gcc}/configure`)} ${args}`.cwd(build("gcc0"));
            await $`make -j16 all-gcc`.cwd(build("gcc0"));
            await $`make install-gcc`.cwd(build("gcc0"));
            await $`make -j16 all-target-libgcc`.cwd(build("gcc0"));
            await $`make install-target-libgcc`.cwd(build("gcc0"));
        }, {after: ["binutils"]})

        hook("compile", "headers", async (ctx, {tar, output, build, rootfs}) => {
            await $`cp ${join(__dirname, ".config")} .config`.cwd(build(v.linux))
            await $`make ARCH=arm64 INSTALL_HDR_PATH=${rootfs()}/usr headers_install`.cwd(build(v.linux))
        }, {after: ['binutils']})

        hook("compile", "glibc0", async (ctx, {tar, build, rootfs}) => {
            await $`mkdir -p ${build("glibc0")}`;
            const args = [
                `--prefix=/usr`,
                '--host=aarch64-linux-gnu',
                '--target=aarch64-linux-gnu',
                '--disable-werror',
                '--without-selinux',
                'libc_cv_forced_unwind=yes',
                `--with-headers=${rootfs()}/usr/include`,
            ]
            await $`${build(`${v.glibc}/configure`)} ${args}`.cwd(build("glibc0"));
            await $`make DESTDIR=${rootfs()} install-bootstrap-headers=yes install-headers`.cwd(build("glibc0"));
            await $`make -j16`.cwd(build("glibc0"));
            await $`make DESTDIR=${rootfs()} install`.cwd(build("glibc0"));
            await $`ln -s ../usr/lib64/crti.o crti.o`.cwd(`${rootfs()}/lib`);
            await $`ln -s ../usr/lib64/crt1.o crt1.o`.cwd(`${rootfs()}/lib`);
            await $`ln -s ../usr/lib64/crtn.o crtn.o`.cwd(`${rootfs()}/lib`);
        }, {after: ["gcc0", "headers"]})

        hook("compile", "gcc", async (ctx, {tar, build, output, rootfs}) => {
            await $`mkdir -p ${build("gcc")}`;
            const args = [
                `--prefix=${output('aarch64-linux-gnu')}`,
                '--target=aarch64-linux-gnu',
                '--enable-languages=c,c++',
                '--enable-shared',
                '--enable-static',
                '--with-cpu=cortex-a53',
                `--with-sysroot=${rootfs()}`,
            ];
            await $`${build(`${v.gcc}/configure`)} ${args}`.cwd(build("gcc"));
            await $`make -j16 all-gcc`.cwd(build("gcc"));
            await $`make -j16 all-target-libgcc`.cwd(build("gcc"));
            await $`make -j16 all-target-libstdc++-v3`.cwd(build("gcc"));

            // rebuild sysroot
            await $`rm -r ${rootfs()}`;
            await $`mkdir ${rootfs()}`;
            await $`mkdir -p ${rootfs('boot')}`;
            await $`mkdir -p ${rootfs('dev')}`;
            await $`mkdir -p ${rootfs('sys')}`;
            await $`mkdir -p ${rootfs('proc')}`;
            await $`mkdir -p ${rootfs('tmp')}`;
            await $`mkdir -p ${rootfs('home')}`;
            await $`mkdir -p ${rootfs('mnt')}`;
            await $`mkdir -p ${rootfs('root')}`;
            await $`mkdir -p ${rootfs('userdata')}`;
            await $`mkdir -p ${rootfs('overlay')}`;
            await $`mkdir -p ${rootfs('usr/lib64')}`;
            // 合并 /lib /lib64 /usr/lib 到 /usr/lib64
            await $`ln -s usr/lib64 lib`.cwd(rootfs());
            await $`ln -s usr/lib64 lib64`.cwd(rootfs());
            await $`ln -s lib64 lib`.cwd(rootfs("usr"));

            await $`make ARCH=arm64 INSTALL_HDR_PATH=${rootfs()}/usr headers_install`.cwd(build(v.linux))
            await $`make install-gcc`.cwd(build("gcc"));
            await $`make install-target-libgcc`.cwd(build("gcc"));
            await $`make install-target-libstdc++-v3`.cwd(build("gcc"));
            await $`cp -r include/* ${rootfs('usr/include')}`.cwd(output('aarch64-linux-gnu/aarch64-linux-gnu'));
            await $`cp -r lib/* ${rootfs('usr/lib64')}`.cwd(output('aarch64-linux-gnu/aarch64-linux-gnu'));
            await $`cp -r lib64/* ${rootfs('usr/lib64')}`.cwd(output('aarch64-linux-gnu/aarch64-linux-gnu'));
        }, {after: ["glibc0"]});

        // 再次编译 glibc
        hook("compile", "glibc", async (ctx, {tar, build, rootfs}) => {
            await $`mkdir -p ${build("glibc")}`;
            const args = [
                `--prefix=/usr`,
                '--host=aarch64-linux-gnu',
                '--target=aarch64-linux-gnu',
                '--enable-kernel=4.9.170',
                `--with-headers=${rootfs()}/usr/include`,
                '--disable-multilib',
                '--disable-werror',
            ]
            await $`${build(`${v.glibc}/configure`)} ${args}`.cwd(build("glibc"));
            await $`make -j16`.cwd(build("glibc"));
            await $`make DESTDIR=${rootfs()} install`.cwd(build("glibc"));
            await $`make LOCALEDEF=localedef DESTDIR=${rootfs()} localedata/install-locales SUPPORTED-LOCALES="C.UTF-8/UTF-8 en_US.UTF-8/UTF-8 zh_CN.UTF-8/UTF-8"`.cwd(build("glibc"));

        }, {after: ["gcc"]});

        hook("compile", "pkgConfig", async (ctx, {tar, build, output, rootfs}) => {
            await $`[ ! -e ${build(v.pkgConfig)} ] || rm -r ${build(v.pkgConfig)}`.cwd(build());
            await $`tar -xf ${tar(`${v.pkgConfig}.tar.gz`)}`.cwd(build());
            const args = [
                `--prefix=${output('aarch64-linux-gnu')}`,
                '--program-prefix=aarch64-linux-gnu-',
                '--disable-host-tool',
                '--with-internal-glib',
                `--with-sysroot=${rootfs()}`,
                `--with-pc-path=${rootfs()}/usr/lib/pkgconfig:${rootfs()}/usr/share/pkgconfig`
            ];
            await $`./configure ${args}`.cwd(build(v.pkgConfig));
            await $`make -j16`.cwd(build(v.pkgConfig));
            await $`make install`.cwd(build(v.pkgConfig));
        }, {after: ["gcc"]})
        hook("clean", "clean", async (ctx, {tar, build, output, rootfs}) => {
            const dirs = ['binutils','gcc','gcc0','glibc0']
            for (let dir of dirs) {
                await $`rm -r ${dir}`.cwd(build());
            }
            await $`[ ! -e ${v.binutils}  ] || rm -r ${v.binutils} `.cwd(build())
            await $`[ ! -e ${v.linux}  ] || rm -r ${v.linux} `.cwd(build())
            await $`[ ! -e ${v.glibc}  ] || rm -r ${v.glibc} `.cwd(build())
            await $`[ ! -e ${v.gcc}  ] || rm -r ${v.gcc} `.cwd(build())
        })

    }
};
export default [toolchain]
