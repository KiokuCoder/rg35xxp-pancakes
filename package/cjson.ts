import {appendFile} from "node:fs/promises";
import {$} from "bun"
import {cmakeArgs, cmakeBuild, mesonBuild, mesonCrossFile, v} from "../scripts/helper.ts";
import type {Pack} from "../scripts/pkg.ts";
import {join} from "path";


const version = 'cJSON-1.7.17';
export const cjson: Pack = {
    name: "cjson",
    version: "1.7.17",
    description: "cJSON is an ultra-lightweight, portable, single-file, simple-as-can-be ANSI-C compliant JSON parser, under MIT license.",
    enable: false,
    dependencies: [],
    async sync({download}): Promise<void> {
        return download({
            url: `https://github.com/DaveGamble/cJSON/archive/refs/tags/v1.7.17.tar.gz`,
            filename: `cJSON-1.7.17.tar.gz`
        })
    },
    async make({rootfs, tar, build}): Promise<void> {
        await $`[ ! -e ${build(version)} ] || rm -r ${build(version)}`.cwd(build());
        await $`tar -xf ${tar(`${version}.tar.gz`)}`.cwd(build());
        await cmakeBuild(build(version), rootfs(), {}, [])
    }
}