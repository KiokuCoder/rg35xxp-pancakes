import type {Pack} from "../scripts/pkg.ts";
import {$} from "bun";

export const rg35xxpFirmware: Pack = {
    name: "rg35xxp-firmware",
    version: "1.0.0",
    description: "",
    enable: true,
    provide: [
        {
            name: "firmware",
            version: "1.0.0"
        }
    ],
    dependencies: [],
    async sync({download}): Promise<void> {
    },
    async make({resource, rootfs, tar, build, env_autoconf}): Promise<void> {
        await $`tar -xf ${resource('firmware.tar.gz')}`.cwd(rootfs());
    }
}
