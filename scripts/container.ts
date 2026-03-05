import {join} from "path";
import type {BunFile} from "bun";
import {workspace} from "./pkg.ts";

const output = join(__dirname, "../output");

export interface Options {
    description?: string
    after?: string[]
    failed?: boolean
    optional?: boolean
}

interface Hook<T> {
    options?: Options,
    name: string,
    call: Callback<T>,
}

export const Phase = ["prepare", "sync", "compile", "pack", "clean", "release"] as const;
export type PhaseType = typeof Phase[number];
export type Register<T> = (p: PhaseType, name: string, w: Callback<T>, options?: Options) => void

export interface Utils {
    // 下载文件到 tar 目录
    download(file: DotTarDotGz): Promise<any>

    // 查询输出目录
    output(...path: string[]): string

    // 查询下载到的源代码压缩包
    tar(...path: string[]): string

    // 查询资源文件
    resource(...path: string[]): string

    // 查询编译目录
    build(...path: string[]): string

    // 根文件系统目录
    rootfs(...path: string[]): string

    // autoconf 编译环境变量
    env_autoconf(env?: Record<string, string>): Record<string, string>

    // check is pack installed
    installed(pack: string): Promise<boolean>
}

export interface Callback<T> {
    (b: T, u: Utils): Promise<any>
}

export interface TestNode<T> {
    install(hook: Register<T>): void
}

export interface DotTarDotGz {
    url: string
    filename: string
    size?: number
    md5?: string
    sha1?: string
}


export class Container<T> {
    private readonly context: T;
    private readonly output: string;

    private readonly hooks: {
        [key in PhaseType]: Hook<T>[]
    };
    private readonly _before: Record<PhaseType, Record<string, ((context: T) => Promise<void>)[]>> = Phase.reduce((acc, cur) => {
        acc[cur] = {};
        return acc
    }, {} as Record<PhaseType, Record<string, ((context: T) => Promise<void>)[]>>);

    private readonly _after: Record<PhaseType, Record<string, ((context: T) => Promise<void>)[]>> = Phase.reduce((acc, cur) => {
        acc[cur] = {};
        return acc
    }, {} as Record<PhaseType, Record<string, ((context: T) => Promise<void>)[]>>);

    constructor(context: T) {
        this.context = context;
        this.output = (context as any)?.output || output;
        this.hooks = Phase.reduce((acc, cur) => {
            acc[cur] = [];
            return acc
        }, {} as {
            [key in PhaseType]: Hook<T>[]
        });
    }

    use(...plugins: TestNode<T>[]) {
        plugins.forEach(p => p.install((p: PhaseType, name: string, w: Callback<T>, options?: Options) => this.hook(p, name, w, options)))
    }

    public before(p: PhaseType, name: string, w: (context: T) => Promise<void>): Container<T> {
        if (!this._before[p][name]) {
            this._before[p][name] = []
        }
        this._before[p][name].push(w);
        return this;
    }

    public after(p: PhaseType, name: string, w: (context: T) => Promise<void>): Container<T> {
        if (!this._after[p][name]) {
            this._after[p][name] = []
        }
        this._after[p][name].push(w);
        return this;
    }

    async start(): Promise<any> {
        const all: [PhaseType, Hook<T>[]][] = Phase.map(p => [p, this.sort(this.hooks[p])])
        for (const [p, hooks] of all) {
            await this.execute(this.context, p, hooks)
        }
        return Promise.resolve()
    }

    private hook(p: PhaseType, name: string, w: Callback<T>, options?: Options): void {
        if (this.hooks[p].map(h => h.name).includes(name)) {
            throw `hook ${name} existed`
        }
        this.hooks[p].push({
            call: w, name, options: options
        });
    }

    private async execute(ctx: T, phase: PhaseType, hooks: Hook<T>[]): Promise<any> {
        const wd = this.output;
        console.log(`current work dir: ${wd}`)
        for (const h of hooks) {
            try {
                console.log(`call hook ${h.name} ${h.options?.description ?? ""}`)
                for (let before of this._before[phase][h.name] ?? []) {
                    await before(ctx)
                }
                await h.call(ctx, workspace)
                for (let after of this._after[phase][h.name] ?? []) {
                    await after(ctx)
                }
                console.log(`call hook ${h.name} done`)
            } catch (e: any) {
                if (e.exitCode && e.stderr) {
                    console.error(e.stderr.toString())
                }
                throw e;
            }
        }
        return Promise.resolve()
    }

    private sort(list: Hook<T>[]): Hook<T>[] {
        const installed: string[] = [];
        const result: Hook<T>[] = [];
        let rest = list;
        while (rest.length > 0) {
            let available: Hook<T>[] = [];
            let unavailable: Hook<T>[] = [];
            for (let h of rest) {
                if ((h.options?.after || []).filter(dep => dep != "" && !installed.includes(dep)).length == 0) {
                    available.push(h)
                } else {
                    unavailable.push(h)
                }
            }
            if (available.length < 1) {
                if (unavailable.every(h => h.options?.optional)) {
                    console.log(`skip optional hooks:${unavailable.map(h => h.name).join(" ")}`)
                    break;
                }
                throw `unable to resolve dependencies:${this.deps(unavailable.filter(h => !(h.options?.optional || false))).join(" ")}`
            }
            result.push(...available);
            installed.push(...available.map(h => h.name))
            rest = unavailable
        }
        return result;
    }

    private deps(list: Hook<any>[]): string[] {
        let result = new Set<string>();
        list.map(h => h.options?.after || []).forEach(arr => {
            arr.forEach(name => result.add(name))
        })
        let array: string[] = [];
        result.forEach(r => array.push(r));
        return array;
    }

}


async function md5(file: BunFile): Promise<string> {
    const md5 = new Bun.CryptoHasher("md5");
    md5.update(await file.arrayBuffer());
    return md5.digest("hex");
}

const mirrors: { [key: string]: string } = {
    "https://cdn.kernel.org/pub/linux/kernel/": "https://mirrors.tuna.tsinghua.edu.cn/kernel/"
};

function cdn(url: string): string {
    for (const prefix in mirrors) {
        if (url.startsWith(prefix)) {
            return mirrors[prefix] + url.substring(prefix.length);
        }
    }
    return url;
}
