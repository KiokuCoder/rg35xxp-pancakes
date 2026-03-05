import {$} from "bun"
import type {Pack} from "../scripts/pkg.ts";

export const rgtv: Pack = {
    name: "rgtv",
    version: "0.1.0",
    description: "IPTV player for RG40XXH, RG35XX SP, RG35XX PLUG, RG35XXH",
    enable: false,
    dependencies: [
        {name: "ffmpeg", version: "*", required: true},
        {name: "sdl2", version: "*", required: true},
    ],
    async sync({download}): Promise<void> {
        return download({
            url: `https://github.com/zwh8800/RGTV/archive/refs/tags/v0.1.0.tar.gz`,
            filename: `RGTV-0.1.0.tar.gz`
        })
    },
    async make({rootfs, output, tar, build, env_autoconf,}): Promise<void> {
        const wd = build('RGTV-0.1.0');
        const tarball = tar('RGTV-0.1.0.tar.gz');
        await $`[ ! -e ${wd} ] || rm -r ${wd}`.cwd(build());
        await $`tar -xf ${tarball}`.cwd(build());
        const env = {
            ...process.env,
            PKG_CONFIG_PATH: "",
            PKG_CONFIG_SYSROOT_DIR: rootfs(),
            PKG_CONFIG_LIBDIR: `${rootfs('/usr/lib64/pkgconfig')}:${rootfs('/usr/share/pkgconfig')}`,
            CC: output('aarch64-linux-gnu/bin/aarch64-linux-gnu-gcc'),
            LD: output('aarch64-linux-gnu/bin/aarch64-linux-gnu-ld'),
            GOOS: "linux",
            GOARCH: "arm64",
            CGO_ENABLED: "1",
            CGO_LDFLAGS: `--sysroot=${rootfs()}`,
            CGO_CFLAGS: `--sysroot=${rootfs()}`
        };
        await $`go build .`.env(env).cwd(wd);
        await $`mkdir -p ${rootfs('root/apps/RGTV')}`.cwd(wd);
        await $`install -Dm 0644 RGTV.png ${rootfs('root/apps/RGTV/icon.png')}`.cwd(wd);
        await $`install -Dm 0755 RGTV ${rootfs('usr/bin/RGTV')}`.cwd(wd);
        await Bun.write(rootfs('root/apps/RGTV/info.toml'), [
            `name="RGTV"`,
            `cmd="rg35xxp-guard RGTV"`,
            `icon="icon.png"`,
        ].join('\n'));
        await Bun.write(rootfs('root/apps/RGTV/tv.json'), JSON.stringify({
            "live_source_url": "",
            "ffmpeg_path": "ffmpeg"
        }));
    }
}
