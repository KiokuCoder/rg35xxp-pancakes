import {$} from "bun"
import {configureMakeMakeInstall} from "../scripts/helper.ts";
import type {Pack} from "../scripts/pkg.ts";


const version = 'curl-8.4.0';
export const curl: Pack = {
    name: "curl",
    version: "8.4.0",
    description: "Curl is a command-line tool and library for transferring data with URL syntax, supporting various protocols.",
    enable: true,
    dependencies: [
        {name: "zlib", version: "*", required: false},
        {name: "openssl", version: "*", required: false},
        {name: "nghttp2", version: "*", required: false} // Optional, for HTTP/2 support
    ],
    async sync({download}): Promise<void> {
        return download({
            url: `https://curl.se/download/${version}.tar.gz`,
            filename: `${version}.tar.gz`
        })
    },
    async make({rootfs, tar, build, env_autoconf}): Promise<void> {
        await $`[ ! -e ${build(version)} ] || rm -r ${build(version)}`.cwd(build());
        await $`tar -xf ${tar(`${version}.tar.gz`)}`.cwd(build());

        const configureArgs = [
            "--sysconfdir=/etc",
            "--with-ca-path=/etc/ssl/certs", // Adjust as needed
            '--with-openssl',
            "--disable-ldap", // Optional, disable if not needed
            "--without-libidn2", // Optional, disable if not needed
        ];

        await configureMakeMakeInstall(build(version), rootfs(), env_autoconf(), configureArgs)
    }
}