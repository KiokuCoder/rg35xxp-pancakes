import type {Pack} from "../scripts/pkg.ts";
import {cmakeBuild} from "../scripts/helper.ts";
import {$} from "bun";

export const leveldb: Pack = {
    name: "leveldb",
    version: "1.23",
    description: "LevelDB is a fast key-value storage library written at Google that provides an ordered mapping from string keys to string values.",
    enable: true,
    dependencies: [],
    async sync({download}): Promise<void> {
        return Promise.resolve();
    },
    async make({rootfs, tar, build, env_autoconf}): Promise<void> {
        const leveldb = build('leveldb')
        await $`[ ! -e leveldb ] || rm -r leveldb`.cwd(build());
        await $`git clone https://github.com/google/leveldb.git`.cwd(build());
        await $`git checkout 578eeb702ec0fbb6b9780f3d4147b1076630d633`.cwd(leveldb);
        await $`git submodule update --init --recursive`.cwd(leveldb);
        await cmakeBuild(leveldb, rootfs(), {}, [
            "-DLEVELDB_BUILD_TESTS=OFF",
            "-DLEVELDB_BUILD_BENCHMARKS=OFF",
            "-DBUILD_SHARED_LIBS=ON",
        ])
    }
}
