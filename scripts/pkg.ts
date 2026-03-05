import type {DotTarDotGz, Utils} from "./container.ts";
import {satisfies} from 'semver';
import {join} from "path";
import {exists, writeFile} from "node:fs/promises";

class Workspace implements Utils {
    private readonly wd: string;

    constructor(wd: string) {
        this.wd = wd;
    }

    resource(...path: string[]): string {
        return join(__dirname, "../resource", ...path)
    }

    output(...path: string[]): string {
        return join(this.wd, ...path)
    }

    build(...path: string[]): string {
        return join(this.wd, "build", ...path)
    }

    async download(file: DotTarDotGz): Promise<string> {
        const path = join(this.wd, "tarball", file.filename)
        const target = Bun.file(path);
        if (await target.exists()) {
            return Promise.resolve(path);
        }

        console.log(`Downloading ${file.url} to ${file.filename}`)
        let data = fetch(file.url).then(res => {
            if (!res.ok) {
                throw new Error(`Download failed: ${file.filename}`);
            }
            return res.blob()
        })
        await Bun.write(target, await data)
        return Promise.resolve(path);
    }

    rootfs(...path: string[]): string {
        return join(this.wd, "rootfs", ...path)
    }

    tar(...path: string[]): string {
        return join(this.wd, "tarball", ...path)
    }

    env_autoconf(env?: Record<string, string>): Record<string, string> {
        return {
            ...process.env,
            'CC': `aarch64-linux-gnu-gcc`,
            'CPP': `aarch64-linux-gnu-cpp`,
            'CXX': `aarch64-linux-gnu-g++`,
            'AR': `aarch64-linux-gnu-ar`,
            'AS': `aarch64-linux-gnu-as`,
            'RANLIB': `aarch64-linux-gnu-ranlib`,
            'OBJCOPY': `aarch64-linux-gnu-objcopy`,
            'OBJDUMP': `aarch64-linux-gnu-objdump`,
            'LD': `aarch64-linux-gnu-ld`,
            'NM': `aarch64-linux-gnu-nm`,
            'STRIP': `aarch64-linux-gnu-strip`,
            'PKG_CONFIG': 'aarch64-linux-gnu-pkg-config',
            PKG_CONFIG_PATH: "",
            PKG_CONFIG_LIBDIR: `${this.wd}/rootfs/usr/lib/pkgconfig:${this.wd}/rootfs/usr/share/pkgconfig`,
            PKG_CONFIG_SYSROOT_DIR: `${this.wd}/rootfs`,
            ...env
        }
    }

    installed(pack: string): Promise<boolean> {
        return exists(join(this.wd, "installed", pack))
    }
}

export const workspace: Utils = new Proxy(new Workspace(join(__dirname, "../output")), {
    get(target, prop, receiver) {
        const value = Reflect.get(target, prop, receiver);
        if (typeof value === 'function') {
            return value.bind(target); // 自动绑定
        }
        return value;
    },
});

export interface Pack {
    name: string
    version: string
    description: string
    enable: boolean
    dependencies: Dependency[]
    provide?: { name: string, version: string }[]

    sync(utils: Utils): Promise<void>

    make(utils: Utils): Promise<void>

    install?(utils: Utils): Promise<void>

    clean?(utils: Utils): Promise<void>
}

export interface Dependency {
    name: string
    version: string
    required: boolean
}

export const packages: Pack[] = [];

export function register(pack: Pack) {
    if (packages.find(p => p.name === pack.name)) {
        throw new Error(`package ${pack.name} already exists`)
    }
    packages.push({...pack})
}

function versionCheck(v: string, need: string): boolean {
    return need === "*" || satisfies(v, need);
}

export function resolve(name?: string): Pack[] {
    const resolved: Pack[] = [];
    const seen: Set<string> = new Set();

    function visit(pack: Pack) {
        if (seen.has(pack.name)) return;
        seen.add(pack.name);

        for (const dep of pack.dependencies) {
            const depPack = packages.find(p => match(dep.name, dep.version, p));
            if (!depPack) {
                if (dep.required) {
                    throw new Error(`Required dependency ${dep.name} not found for package ${pack.name}`);
                }
                continue;
            }
            depPack.enable = true;
            visit(depPack);
        }

        if (pack.enable || pack.name === name) {
            resolved.push(pack);
        }
    }

    if (name) {
        const pack = packages.find(p => p.name === name);
        if (!pack) {
            throw new Error(`Package ${name} not found`);
        }
        visit(pack);
    } else {
        for (const pack of packages) {
            visit(pack);
        }
    }

    return resolved;
}

const cache = join(__dirname, "../output/installed");
const state = {current: "", phase: ""};

export async function install(packs: Pack[]) {
    for (const pack of packs) {
        state.current = pack.name;
        state.phase = "sync";
        await pack.sync.bind(pack)(workspace);
    }

    for (const pack of packs) {
        if (await exists(join(cache, pack.name))) {
            console.log(`Package ${pack.name} already installed.`);
            continue;
        }
        console.log(`==> building ${pack.name} `)
        state.current = pack.name;
        state.phase = "make";
        await pack.make.bind(pack)(workspace);
        if (pack.install) {
            state.current = pack.name;
            state.phase = "install";
            await pack.install.bind(pack)(workspace);
        }
        await writeFile(join(cache, pack.name), "");
    }

    for (const pack of packs) {
        state.current = pack.name;
        state.phase = "clean";
        pack.clean && await pack.clean.bind(pack)(workspace);
    }
    state.current = "";
    state.phase = "";

    console.log("All packages have been installed.");
}

export function current() {
    return {...state}
}

function match(name: string, version: string, p: Pack): boolean {
    if (p.name === name && versionCheck(p.version, version)) return true;
    if (Array.isArray(p.provide)) {
        for (let pp of p.provide) {
            if (pp.name === name && versionCheck(pp.version, version)) return true;
        }
    }
    return false;
}
