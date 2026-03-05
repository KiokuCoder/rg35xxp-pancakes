import {$} from "bun"
import {configureMakeMakeInstall} from "../scripts/helper.ts";
import type {Pack} from "../scripts/pkg.ts";

const version = 'pixman-0.42.2';
export const pixman: Pack = {
    name: "pixman",
    version: "0.42.2",
    description: "Pixman is a low-level software library for pixel manipulation.",
    enable: false,
    dependencies: [],
    async sync({download}): Promise<void> {
        return download({
            url: `https://www.cairographics.org/releases/${version}.tar.gz`,
            filename: `${version}.tar.gz`
        })
    },
    async make({resource, rootfs, tar, build, env_autoconf}): Promise<void> {
        await $`[ ! -e ${build(version)} ] || rm -r ${build(version)}`.cwd(build());
        await $`tar -xf ${tar(`${version}.tar.gz`)}`.cwd(build());

        const args = [
            '--disable-gtk', // optional, depending on your needs
            '--disable-docs' // optional
        ];
        await configureMakeMakeInstall(build(version), rootfs(), env_autoconf(), args)
    }
}
