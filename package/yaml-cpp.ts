import {$} from "bun"
import type {Pack} from "../scripts/pkg.ts";
import {cmakeBuild} from "../scripts/helper.ts";

export const yamlCPP: Pack = {
    name: "yaml-cpp",
    version: "0.8.0",
    description: "YAML parser and emitter in C++",
    enable: false,
    dependencies: [],
    async sync({download}): Promise<void> {
        return download({
            url: `https://github.com/jbeder/yaml-cpp/archive/refs/tags/0.8.0.tar.gz`,
            filename: `yaml-cpp-0.8.0.tar.gz`
        })
    },
    async make({rootfs, tar, build, env_autoconf}): Promise<void> {
        const wd = build('yaml-cpp-0.8.0');
        await $`[ ! -e ${wd} ] || rm -r ${wd}`;
        await $`tar -xf ${tar('yaml-cpp-0.8.0.tar.gz')}`.cwd(build());
        await cmakeBuild(wd, rootfs(), {}, ["-DYAML_BUILD_SHARED_LIBS=ON"])
    }
}
