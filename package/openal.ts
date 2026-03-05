import type { Pack } from "../scripts/pkg.ts";
import { cmakeBuild } from "../scripts/helper.ts";
import { $ } from "bun";

const version = 'openal-soft-1.24.2';
export const openal: Pack = {
    name: "openal",
    version: "1.24.2",
    description: "OpenAL Soft is a software implementation of the OpenAL 3D audio API.",
    enable: true,
    dependencies: [
        {
            name: "alsa",
            version: "*",
            required: true
        }
    ],
    async sync({ download }): Promise<void> {
        return download({
            url: `https://openal-soft.org/openal-releases/${version}.tar.bz2`,
            filename: `${version}.tar.bz2`
        })
    },
    async make({ rootfs, tar, build }): Promise<void> {
        await $`[ ! -e ${build(version)} ] || rm -r ${build(version)}`.cwd(build());
        await $`tar -xf ${tar(`${version}.tar.bz2`)}`.cwd(build());
        await cmakeBuild(build(version), rootfs(), {}, [
            "-DALSOFT_EXAMPLES=OFF",
            "-DALSOFT_UTILS=OFF",
        ])
    }
}
