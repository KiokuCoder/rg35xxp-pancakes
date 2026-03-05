import {Container, type Register, type TestNode} from "./scripts/container.ts";
import {toolchain} from "./scripts/toolchain.ts";
import type {Context} from "./scripts/config.ts";
import {join} from "path";
import {mkdir} from "node:fs/promises";

async function mkdirp(path: string): Promise<void> {
    const wd = Bun.file(path)
    if (await wd.exists()) {
        return
    }
    await mkdir(path, {recursive: true})
}

const init: TestNode<Context> = {
    install(hook: Register<Context>): void {
        hook("prepare", "init", async (ctx, {output, tar, build}) => {
            await mkdirp(tar())
            await mkdirp(build())
            await mkdirp(output("installed"))
            process.chdir(output())
        }, {after: []})
    }
};

const ctx: Context = {dir: join(process.cwd(), "temp"), prefix: "/opt/aarch64-linux-gnu"}
const container = new Container(ctx);

container.use(init, toolchain);

process.env['PATH'] = `${join(__dirname, "output/aarch64-linux-gnu/bin")}:${process.env['PATH']}`
process.env['LANG'] = "en_US.UTF-8"
process.env['LC_CTYPE'] = "en_US.UTF-8"
await container.start().then(() => {
    console.log("done")
});

