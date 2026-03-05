import {$} from "bun"
import type {Pack} from "../scripts/pkg.ts";

export const notoCJK: Pack = {
    name: "noto-cjk",
    version: '1.0.0',
    description: "Google Noto Sans CJK fonts",
    enable: true,
    dependencies: [
        {
            name: "fontconfig",
            version: "2.14.2",
            required: true
        }
    ],
    async sync({download}): Promise<void> {
        return download({
            url: `https://github.com/notofonts/noto-cjk/releases/download/Serif2.003/01_NotoSerifCJK.ttc.zip`,
            filename: `01_NotoSerifCJK.ttc.zip`
        })
    },
    async make({rootfs, tar, build}): Promise<void> {
        const d =build('01_NotoSerifCJK.ttc');
        await $`[ ! -e ${d} ] || rm -r ${d}`.cwd(build());
        await $`mkdir -p ${d}`;
        await $`unzip -o ${tar('01_NotoSerifCJK.ttc.zip')} -d ${d}`;

        const fontDir = `${rootfs()}/usr/share/fonts/noto-cjk`;
        await $`mkdir -p ${fontDir}`;
        await $`cp ${d}/*.ttc ${fontDir}`;
    }
}
