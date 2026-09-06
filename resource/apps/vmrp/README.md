# vmrp

基于 unicorn 引擎的 mrp(mythroad) 模拟器，上游是 <https://github.com/vmrp/vmrp>。

上游只提供 windows(mingw32) 和 wasm 两套构建脚本，这里按 `wasm/Makefile` 的源码列表
整理出交叉编译到掌机(aarch64)所需的最小集合，加一个 `Makefile` 交叉编译到 aarch64，
由 `package/vmrp.ts` 调用。**收录的源文件都和上游保持一致**，只打了下面「相对上游的改动」
里列出的补丁，方便后续跟上游同步；上游的 win32 分支虽然在掌机上永远不会被编译，
也原样保留。

## 来源

- 仓库：<https://github.com/vmrp/vmrp>
- commit：`ffe01e9fc203a830ad55b6b9636500e0fdc971b4`

## 目录说明

| 路径                 | 说明                                                                     |
|--------------------|------------------------------------------------------------------------|
| `*.c`              | 编译用到的源文件，和上游根目录同名文件对应，`sound.c` 除外(见下)                          |
| `header/`          | 上游 `header/` 全量拷贝，外加 `sound.h`                                         |
| `mythroad/include/` | `bridge.c` 需要的 mythroad 接口头文件，**只有头文件**，mythroad 本身不参与编译             |
| `fs/`              | 运行时数据，取自上游 `wasm/dist/fs/`，是编译好的二进制启动文件，直接拷贝到掌机 `/root/apps/vmrp/` |
| `Makefile`         | 交叉编译脚本，替换了上游针对 mingw32 的根 `Makefile`                                  |

未收录的上游内容：`mythroad/`(客户机侧固件源码，用预编译的 `fs/` 代替)、`debug.c`
(只在 `-DDEBUG` 下使用，依赖 capstone)、`windows/`、`wasm/`、`mrc/`、`tool/`、`doc/`。

`fs/cfunction.ext` 和 `fs/mythroad/` 是 vmrp 运行的必要文件，vmrp 按**当前目录**查找它们，
所以 `main.c` 里在找不到 `cfunction.ext` 时会 `chdir` 到 `VMRP_HOME`。

## 相对上游的改动

除了下面列出的，其余文件（`fileLib.c`/`network.c`/`rbtree.c`/`utils.c`/
`header/posix_sockets.h` 等）和上游逐字节相同。

1. `header/utils.h`：上游只写死了 windows/wasm 的 unicorn 头文件路径，linux 下补一条
   `#include <unicorn/unicorn.h>`，用系统安装的版本。
2. `memory.c`：`initMemoryManager()` 把宿主机指针截断成 `uint32`，在 aarch64 上会崩溃，改用 `uintptr_t`。
3. `vmrp.c`：unicorn 在 64 位主机上默认申请 1GiB 的 TCG 翻译缓冲区，掌机内存不够会启动失败，
   改成 16MiB（mrp 代码段只有 1MiB）。
4. `main.c`：
   - 掌机的实体按键是手柄设备(ANBERNIC-keys)，不产生键盘事件，补上 `SDL_GameController` 映射；
   - `main()` 丢掉了 `startVmrp()` 的返回值，初始化失败后仍然进事件循环，第一次按键会从 PC=0
     取指崩溃，改成失败即退出；
   - 找不到 `cfunction.ext` 时自动 `chdir` 到 `VMRP_HOME`，省掉外面的 wrapper 脚本；
   - `guiDrawBitmap()` 里给 `SDL_GetWindowSurface()` 加了返回 NULL 的判断，避免空指针解引用；
   - 初始化 SDL 时带上 `SDL_INIT_AUDIO`，并调用 `soundInit()`/`soundDeinit()`；
   - 加了 `--rotate` 参数和画面居中放大，见下面的「画面旋转」。
5. `sound.c` + `header/sound.h`：**上游没有的新文件**，见下面的「声音」。
6. `bridge.c`：把 `mr_playSound()`/`mr_stopSound()` 的原生分支接到 `sound.c`，
   并把 `mr_table` 里这两项从 `NULL` 补成实现——原来 ext 类型的 mrp 一放声音就会打印
   `Not yet implemented function` 然后 `exit(1)`。


## 画面旋转

客户机画面固定是 240x320 的竖屏，掌机屏幕是 640x480 的横屏。上游按原尺寸 1:1 写窗口
surface，而掌机的 sunxi 后端会忽略 `SDL_CreateWindow()` 请求的尺寸直接开全屏，结果画面
只占左上角 240x320 一小块。竖着放最多也只能 1 倍（480/320=1），顺时针转 90 度之后是
320x240，正好整数放大 2 倍铺满 640x480，做到点对点。

```
vmrp --rotate 90     # 或 -r 90 / --rotate=90，取值 0/90/180/270，顺时针，默认 0
```

`package/vmrp.ts` 生成的 `info.toml` 里默认就带 `--rotate 90`。

放大倍率和居中偏移由 `updateViewport()` 按窗口实际大小算：等比整数放大到能放下的最大倍数，
居中，四周留黑边（只在窗口尺寸变化时刷一次）。`viewMap()` 把客户机坐标换成窗口坐标，
`viewUnmap()` 是它的逆变换，鼠标/触摸事件走这条路换回客户机坐标。转 0 度时倍率是 1，
和上游的区别只是画面被居中了。

画面转了方向键也要跟着转，`rotateKey()` 把方向键往反方向转回去（非方向键原样透传），
`keyEvent()`/`controllerEvent()` 里所有 `MR_KEY_*` 都改走 `sendKey()`。以 90 度为例，
客户机的「上」朝向屏幕右边，所以：

| 实体按键 | 客户机按键 |
|------|-------|
| 右    | 上     |
| 下    | 右     |
| 左    | 下     |
| 上    | 左     |

也就是把掌机逆时针转 90 度（左边缘朝上）拿在手里，方向就和画面对上了。习惯反方向的话
用 `--rotate 270`，画面和按键一起换到另一边。

## 声音

上游只在 wasm 版实现了声音（`mr_playSound()` 转给 JS 的 `js_playSound()`），
windows/linux 原生版是空实现，直接 `return MR_SUCCESS`，所以掌机上一点声音都没有。
`sound.c` 按 mrp 的声音类型各接一个解码器，统一转成 44100Hz / 16bit / 立体声之后混音：

| 类型              | 实现                                                   |
|-----------------|------------------------------------------------------|
| `MR_SOUND_MIDI` | libADLMIDI，OPL3(YMF262) FM 合成，音色表编译在库里，不需要额外的音色库文件 |
| `MR_SOUND_WAV`  | SDL 自带的 `SDL_LoadWAV_RW()`                           |
| `MR_SOUND_MP3`  | libmpg123                                            |
| `MR_SOUND_PCM`  | 8K 16bit 单声道裸数据，只做重采样                                |
| `MR_SOUND_AMR`  | 没有解码器，返回 `MR_FAILED`                                 |

`mr_stopSound()` 是按类型停的，所以每种类型各占一路，同时响的互不影响
（典型用法是 MIDI 放背景音乐、WAV 放音效）。MIDI 选 DosBox 那个轻量的 OPL3 实现，
默认的 Nuked 精度高但太吃 CPU，一边跑 unicorn 一边合成会卡；`package/libadlmidi.ts`
里也只编了 DosBox 一个。打不开音频设备只是没声音，不影响模拟器继续跑。

## 同步上游

源码是**打过补丁**的，升级上游版本时先拿到差异再合并：

```bash
git clone https://github.com/vmrp/vmrp.git /tmp/vmrp
git -C /tmp/vmrp checkout ffe01e9fc203a830ad55b6b9636500e0fdc971b4
diff -ru /tmp/vmrp/header header
for f in *.c; do diff -u "/tmp/vmrp/$f" "$f"; done
```

## 单独编译

```bash
make            # 产物 build/vmrp
make clean
```

依赖 rootfs 里已经装好的 sdl2 / zlib / unicorn / libADLMIDI / libmpg123，需要 `PKG_CONFIG_LIBDIR`、
`PKG_CONFIG_SYSROOT_DIR` 指向 rootfs，直接跑 `bun pack-one.ts vmrp` 更省事。
