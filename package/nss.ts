import {appendFile} from "node:fs/promises";
import {$, env} from "bun"
import {cmakeArgs, cmakeBuild, mesonBuild, mesonCrossFile, v} from "../scripts/helper.ts";
import type {Pack} from "../scripts/pkg.ts";
import {join} from "path";


export const nss: Pack = {
    name: "nss",
    version: "3.96",
    description: "Network Security Services",
    enable: false,
    dependencies: [
        {
            name: "zlib",
            version: "*",
            required: true
        },
    ],
    async sync({download}): Promise<void> {
        return download({
            url: `https://ftp.mozilla.org/pub/security/nss/releases/NSS_3_100_RTM/src/nss-3.100-with-nspr-4.35.tar.gz`,
            filename: `nss-3.100-with-nspr-4.35.tar.gz`
        })
    },
    async make({rootfs, tar, build, env_autoconf}): Promise<void> {
        const version = 'nss-3.100';
        await $`[ ! -e ${build(version)} ] || rm -r ${build(version)}`.cwd(build());
        await $`tar -xf ${tar(`nss-3.100-with-nspr-4.35.tar.gz`)}`.cwd(build());
        await $`make`.env(env_autoconf()).cwd(build(version, 'nss'));
    }
}