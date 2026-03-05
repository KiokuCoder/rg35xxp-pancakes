import type {Pack} from "../scripts/pkg.ts";
import {$} from "bun";

export const adb: Pack = {
    name: "adb",
    version: "0.0.1",
    description: "",
    enable: true,
    dependencies: [],
    async sync({download}): Promise<void> {
    },
    async make({resource, rootfs, tar, build, env_autoconf}): Promise<void> {
        await $`[ ! -e adb-0.0.1 ] || rm -r adb-0.0.1`.cwd(build());
        await $`tar -xf ${resource('adb-0.0.1.tar.gz')}`.cwd(build());
        await $`make`.env(env_autoconf()).cwd(build('adb-0.0.1/src'));
        await $`install -Dm 0755 adbd ${rootfs()}/usr/bin/adbd`.cwd(build('adb-0.0.1/src'));
        await $`ln -s /usr/bin/bash ${rootfs()}/bin/adb_shell`.cwd(build('adb-0.0.1/src'));
    }
}
