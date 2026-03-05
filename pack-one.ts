#!/usr/bin/env bun
import {current, install, resolve} from './scripts/pkg';
import "./package/index"
import {join} from "path";

const packageName = process.argv[2];
if (!packageName) {
    console.error("Please provide a package name.");
    process.exit(1);
}

process.env['PATH'] = `${join(__dirname, "output/aarch64-linux-gnu/bin")}:${process.env['PATH']}`
process.env['LANG'] = "en_US.UTF-8"
process.env['LC_CTYPE'] = "en_US.UTF-8"
install(resolve(packageName)).catch(err => {
    const state = current();
    console.error(`error in packing package ${state.current} (phase: ${state.phase}): `);
    console.error(err);
    if (isShellError(err)) {
        const msg = `error in packing package ${state.current} (phase: ${state.phase}, exitCode: ${err.exitCode}):\n\n${err.stderr} `;
        const file = join(__dirname,`output/${new Date().toISOString()}.log`);
        console.log(`log file save to: ${file}`)
        return Bun.write(file, msg)
    }
});

function isShellError(err: any): boolean {
    return err !== undefined && err !== null && (typeof err.exitCode === "number")
}