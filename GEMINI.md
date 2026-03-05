我正在使用 **TypeScript** 语言制作一个类似 buildroot 的编译软件。主要用到的方法接口如下：

```typescript

export interface Pack {
    name: string
    version: string
    description: string
    enable: boolean
    dependencies: Dependency[]

    sync(utils: Utils): Promise<void>

    make(utils: Utils): Promise<void>

}

export interface Dependency {
    name: string
    version: string
    required: boolean
}
export interface Utils {
   // 下载文件到 tar 目录
   download(file: DotTarDotGz): Promise<any>

   // 查询输出目录
   output(...path: string[]): string

   // 查询下载到的源代码压缩包
   tar(...path: string[]): string

   // 查询编译目录
   build(...path: string[]): string

   // 根文件系统目录
   rootfs(...path: string[]): string

   // cmake 编译环境变量
   env_cmake(env?: Record<string, string>): Record<string, string>

   // autoconf 编译环境变量
   env_autoconf(env?: Record<string, string>): Record<string, string>
}

export interface DotTarDotGz {
    url: string
    filename: string
    size?: number
    md5?: string
    sha1?: string
}

export async function configureMakeMakeInstall(wd: string, rootfs: string, env: Record<string, string>, args: string[]) {
}

export async function mesonBuild(wd: string, rootfs: string, args: string[]) {
}

export function cmakeBuild(wd: string, rootfs: string, env: Record<string, string>, args: string[]) {
}
```

里面有一个 **after** 参数表示软件编译时候需要用到的依赖，你可以根据实际情况填写。
`Utils` 里面的 `tar`/`build` 是用来计算文件、目录的路径。`download` 用来下载源代码文件。
通常每个软件包都需要单通过一个 `[name].ts` 的文件描述一次，比如编译 **zlib** 的描述文件内容如下：

```typescript
const version = 'zlib-1.3.1';
export const zlib: Pack = {
    name: "zlib",
    version: "1.3.1",
    description: "zlib is a software library used for data compression as well as a data format.",
    enable: true,
    dependencies: [],
    async sync({download}): Promise<void> {
        return download({
            url: `https://zlib.net/${version}.tar.gz`,
            filename: `${version}.tar.gz`
        })
    },
    async make({rootfs, tar, build, env_cmake, env_autoconf}): Promise<void> {
        await $`[ ! -e ${build(version)} ] || rm -r ${build(version)}`.cwd(build());
        await $`tar -xf ${tar(`${version}.tar.gz`)}`.cwd(build());
        await configureMakeMakeInstall(build(version), rootfs(), env_autoconf(), [])
    }
}
```

根据软件使用的构建工具 autoconf/cmake/meson 调用不同的方法，如果软件使用独特的编译脚本，那么根据这个软件的编译流程进行*
*交叉编译**。
比如**busybox**，它描述文件如下：

```typescript
const version = 'busybox-1.36.1';
export const busybox: Pack = {
    name: "busybox",
    version: "1.36.1",
    description: "BusyBox is a software suite that provides several Unix utilities in a single executable file.",
    enable: true,
    dependencies: [
        {
            name: "zlib",
            version: "1.0.0",
            required: true
        }
    ],
    async sync({download}): Promise<void> {
        return download({
            url: `https://busybox.net/downloads/${version}.tar.bz2`,
            filename: `${version}.tar.bz2`
        })
    },
    async make({rootfs, tar, build}): Promise<void> {
        await $`[ ! -e ${build(version)} ] || rm -r ${build(version)}`.cwd(build());
        await $`tar -xf ${tar(`${version}.tar.bz2`)}`.cwd(build());
        const args = ["-j16", "CROSS_COMPILE=aarch64-linux-gnu-"]
        await $`make ${args} defconfig`.cwd(build(version));
        await $`make ${args} `.cwd(build(version));
        await $`make ${args} install CONFIG_PREFIX=${rootfs()}`.cwd(build(version));
    }
}
```

### 目录说明

- `output` 编辑过程文件，请勿读取
- `package` 软件包存放路径
- `package/index.ts` 新增的软件包，需要注册到系统里面（自动打包、依赖管理）
- `package/all.ts` 一个虚拟包，描述默认编译哪些组件
- `package/sdl2.ts` SDL2 编译脚本
- `resource` 一些内置静态资源，主要是配置文件或是其他二进制文件
- `resource/apps` 存放 `package/rg35xxp-apps.ts` 包的源代码 （Rust 项目）
- `scripts` 框架核心编译脚本，
- `scripts/toolchain.ts` 核心工具链 ( gcc/g++ ) 制作脚本
- `scripts/pkg.ts` 软件包打包脚本
- `scripts/helper.ts` 编译辅助函数
- `pack-one.ts` 打包单个软件包，我开发的时候通常使用这个脚本进行测试
