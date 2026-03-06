import {$} from "bun"
import type {Pack} from "../../scripts/pkg.ts";
import {configureMakeMakeInstall} from "../../scripts/helper.ts";

const version = 'yaml-0.2.5';
export const yaml: Pack = {
    name: "yaml",
    version: "0.4.2",
    description: "LibYAML is a YAML parser and emitter written in C.",
    enable: false,
    dependencies: [],
    async sync({download}): Promise<void> {
        return download({
            url: `https://pyyaml.org/download/libyaml/${version}.tar.gz`,
            filename: `${version}.tar.gz`
        })
    },
    async make({rootfs, tar, build, env_autoconf}): Promise<void> {
        await $`[ ! -e ${build(version)} ] || rm -r ${build(version)}`.cwd(build());
        await $`tar -xf ${tar(`${version}.tar.gz`)}`.cwd(build());
        await configureMakeMakeInstall(build(version), rootfs(), env_autoconf(), [])
    }
}