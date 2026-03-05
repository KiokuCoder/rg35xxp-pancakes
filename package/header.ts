import {appendFile} from "node:fs/promises";
import {$} from "bun"
import {cmakeArgs, cmakeBuild, mesonBuild, mesonCrossFile, v} from "../scripts/helper.ts";
import type {Pack} from "../scripts/pkg.ts";
import {join} from "path";
