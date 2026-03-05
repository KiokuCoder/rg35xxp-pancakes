import type {Pack} from "../scripts/pkg.ts";
import {$} from "bun";

export const skeleton: Pack = {
    name: "skeleton",
    version: "1.0.0",
    description: "",
    enable: true,
    provide: [],
    dependencies: [],
    async sync({download}): Promise<void> {
    },

    async make({resource, rootfs}): Promise<void> {
        for (let d of ['dev', 'etc', 'media', 'mnt', 'opt', 'proc', 'root', 'sys', 'tmp', 'usr']) {
            await $`mkdir -p ${rootfs(d)}`;
        }
        await $`cp -r ${resource('root')}/* ${rootfs()}`;
    }
}
