import {$} from "bun"
import type {Pack} from "../scripts/pkg.ts";
import {cmakeBuild} from "../scripts/helper.ts";


const version = 'x265_3.6';
export const x265: Pack = {
    name: "x265",
    version: "3.6",
    description: "",
    enable: false,
    dependencies: [],
    async sync({download}): Promise<void> {
        return download({
            url: `https://download.videolan.org/pub/videolan/x265/${version}.tar.gz`,
            filename: `${version}.tar.gz`
        })
    },

    async make({resource, rootfs, tar, build, env_autoconf}): Promise<void> {
        await $`[ ! -e ${build(version)} ] || rm -r ${build(version)}`.cwd(build());
        await $`tar -xf ${tar(`${version}.tar.gz`)}`.cwd(build());
        const env: Record<string, string> = {
            CMAKE_SYSTEM_NAME: "Linux",
            CMAKE_SYSTEM_PROCESSOR: "aarch64",
            CMAKE_FIND_ROOT_PATH: rootfs(),
            CMAKE_INSTALL_PREFIX: '/usr',
            CMAKE_FIND_ROOT_PATH_MODE_LIBRARY: "ONLY",
            CMAKE_FIND_ROOT_PATH_MODE_INCLUDE: "ONLY",
            CMAKE_C_COMPILER: `aarch64-linux-gnu-gcc`,
            CMAKE_CXX_COMPILER: `aarch64-linux-gnu-g++`,
            CMAKE_AR: `aarch64-linux-gnu-ar`,
            CMAKE_AS: `aarch64-linux-gnu-as`,
            CMAKE_RANLIB: `aarch64-linux-gnu-ranlib`,
            CMAKE_OBJCOPY: `aarch64-linux-gnu-objcopy`,
            CMAKE_OBJDUMP: `aarch64-linux-gnu-objdump`,
            CMAKE_LINKER: `aarch64-linux-gnu-ld`,
            CMAKE_NM: `aarch64-linux-gnu-nm`,
            CMAKE_STRIP: `aarch64-linux-gnu-strip`,
            PKG_CONFIG_EXECUTABLE: 'aarch64-linux-gnu-pkg-config',
        }
        // await $`cmake -DCMAKE_TOOLCHAIN_FILE="crosscompile.cmake" -DCMAKE_BUILD_TYPE=Release ../../source`.env(env).cwd(build(version, "build/aarch64-linux"));
        // await $`make -j16`.cwd(build(version, "build/aarch64-linux"));
        // await $`make install DESTDIR=${rootfs()}`.cwd(build(version, "build/aarch64-linux"));
        await cmakeBuild(build(version, 'source'), rootfs(), {}, [])
    }
};