import {$} from "bun";
import type {Pack} from "../scripts/pkg.ts";

export const t507_gpu_drivers: Pack = {
    name: "t507_gpu_drivers",
    version: "1.0.0",
    description: "Simple DirectMedia Layer is a cross-platform development library designed to provide low level access to audio, keyboard, mouse, joystick, and graphics hardware via OpenGL and Direct3D.",
    enable: false,
    dependencies: [],
    provide: [
        {
            name: "egl",
            version: "1.4.1",
        },
        {
            name: "gles",
            version: "1.4.0",
        },
        {
            name: "glesv2",
            version: "2.1.0",
        },
        {
            name: "opencl",
            version: "2.1.0",
        }

    ],
    async sync({download}): Promise<void> {
    },
    async make({rootfs, resource, build, env_autoconf}): Promise<void> {
        const src = build('t507_gpu_drivers');
        await $`[ ! -e ${src} ] || rm -r ${src}`.cwd(build());
        await $`[ -e t507_gpu_drivers ] || git clone --depth 1 https://github.com/knulli-cfw/t507_gpu_drivers.git`.cwd(build());
        await $`patch -p1 < ${resource('patch/mali_native_window.patch')}`.cwd(src)
        await $`cp -r fbdev/mali-g31/aarch64-linux-gnu-7.4.1/bin/* ${rootfs('usr/bin')}`.cwd(src);
        await $`cp -r fbdev/mali-g31/aarch64-linux-gnu-7.4.1/lib64/* ${rootfs('usr/lib64')}`.cwd(src);
        await $`cp -r include/* ${rootfs('usr/include')}`.cwd(src);
        await $`cp -r fbdev/include/* ${rootfs('usr/include')}`.cwd(src);
        await $`ln -s libOpenCL.so.2.1.0 usr/lib64/libOpenCL.so || true`.cwd(rootfs())
        await Bun.write(rootfs("usr/lib/pkgconfig/glesv2.pc"), `prefix=/usr
exec_prefix=\${prefix}
libdir=\${exec_prefix}/lib64
includedir=\${prefix}/include

Name: GLESv2
Description: OpenGL ES 2.1 library
Version: 2.1.0
Requires:
Libs: -L\${libdir} -lGLESv2 -lmali
Cflags: -I\${includedir}`)
        await Bun.write(rootfs("usr/lib/pkgconfig/egl.pc"), `prefix=/usr
exec_prefix=\${prefix}
libdir=\${exec_prefix}/lib64
includedir=\${prefix}/include

Name: EGL
Description: Embedded Graphics Library
Version: 1.4.1
Requires:
Libs: -L\${libdir} -lEGL -lmali
Cflags: -I\${includedir}`)
        await Bun.write(rootfs("usr/lib/pkgconfig/OpenCL.pc"), `prefix=/usr
exec_prefix=\${prefix}
libdir=\${exec_prefix}/lib64
includedir=\${prefix}/include/CL

Name: OpenCL
Description: OpenCL library
Version: 2.1.0
Requires:
Libs: -L\${libdir} -lOpenCL -lmali
Cflags: -I\${includedir}`)
    }
}
