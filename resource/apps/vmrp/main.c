#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "./header/bridge.h"
#include "./header/vmrp.h"
#include "./header/memory.h"
#include "./header/sound.h"

#ifdef _WIN32
// #ifdef __x86_64__
// #include "./windows/SDL2-2.0.10/x86_64-w64-mingw32/include/SDL2/SDL.h"
// #elif __i386__
#include "./windows/SDL2-2.0.10/i686-w64-mingw32/include/SDL2/SDL.h"
// #endif
#else
#include <SDL2/SDL.h>
#endif

#ifdef __EMSCRIPTEN__
#include <emscripten.h>
#endif

// 交叉编译到掌机时由 -DVMRP_HOME=... 指定数据目录，见 package/vmrp.ts
#ifdef VMRP_HOME
#include <errno.h>
#include <unistd.h>
#endif

#define MOUSE_DOWN 2
#define MOUSE_UP 3
#define MOUSE_MOVE 12

// 客户机画面固定 240x320(竖屏)，掌机屏幕是 640x480(横屏)，1:1 画只占左上角一小块。
// 顺时针转 90 度之后是 320x240，正好整数放大 2 倍铺满 640x480，做到点对点。
// 角度由命令行 --rotate 指定，只接受 0/90/180/270，顺时针。
static int screenRotate = 0;
static int viewScale = 1;         // 客户机像素放大的整数倍率
static int viewX = 0, viewY = 0;  // 旋转放大后画面在窗口里的左上角
static int viewW = -1, viewH = -1;  // 上次算 viewport 时的窗口大小

// 旋转放大后画面的尺寸
static void viewSize(int *w, int *h) {
    bool swap = (screenRotate == 90 || screenRotate == 270);
    *w = swap ? SCREEN_HEIGHT : SCREEN_WIDTH;
    *h = swap ? SCREEN_WIDTH : SCREEN_HEIGHT;
}

// 按窗口实际大小重算等比整数放大倍率和居中偏移。掌机上窗口总是屏幕大小
// (sunxi 后端强制全屏，忽略 SDL_CreateWindow() 请求的尺寸)。
// 返回 true 表示窗口尺寸变了，调用者要重新把黑边刷一遍。
static bool updateViewport(SDL_Surface *surface) {
    if (surface->w == viewW && surface->h == viewH) {
        return false;
    }
    viewW = surface->w;
    viewH = surface->h;

    int w, h;
    viewSize(&w, &h);
    viewScale = viewW / w;
    if (viewH / h < viewScale) viewScale = viewH / h;
    if (viewScale < 1) viewScale = 1;  // 窗口比画面还小，只能裁掉一部分

    viewX = (viewW - w * viewScale) / 2;
    viewY = (viewH - h * viewScale) / 2;
    if (viewX < 0) viewX = 0;
    if (viewY < 0) viewY = 0;
    return true;
}

// 客户机坐标 -> 窗口坐标(该客户机像素放大后的左上角)
static void viewMap(int32_t gx, int32_t gy, int32_t *wx, int32_t *wy) {
    switch (screenRotate) {
        case 90:  // 顺时针：客户机左上角转到右上角
            *wx = viewX + (SCREEN_HEIGHT - 1 - gy) * viewScale;
            *wy = viewY + gx * viewScale;
            break;
        case 180:
            *wx = viewX + (SCREEN_WIDTH - 1 - gx) * viewScale;
            *wy = viewY + (SCREEN_HEIGHT - 1 - gy) * viewScale;
            break;
        case 270:  // 逆时针：客户机左上角转到左下角
            *wx = viewX + gy * viewScale;
            *wy = viewY + (SCREEN_WIDTH - 1 - gx) * viewScale;
            break;
        default:
            *wx = viewX + gx * viewScale;
            *wy = viewY + gy * viewScale;
            break;
    }
}

// viewMap() 的逆变换，把鼠标/触摸的窗口坐标换回客户机坐标
static void viewUnmap(int32_t wx, int32_t wy, int32_t *gx, int32_t *gy) {
    int32_t px = (wx - viewX) / viewScale;
    int32_t py = (wy - viewY) / viewScale;
    // 落在黑边上时会算出负数或越界，夹到画面内
    int w, h;
    viewSize(&w, &h);
    if (px < 0) px = 0;
    if (py < 0) py = 0;
    if (px >= w) px = w - 1;
    if (py >= h) py = h - 1;

    switch (screenRotate) {
        case 90:
            *gx = py;
            *gy = SCREEN_HEIGHT - 1 - px;
            break;
        case 180:
            *gx = SCREEN_WIDTH - 1 - px;
            *gy = SCREEN_HEIGHT - 1 - py;
            break;
        case 270:
            *gx = SCREEN_WIDTH - 1 - py;
            *gy = px;
            break;
        default:
            *gx = px;
            *gy = py;
            break;
    }
}

// 画面转了方向键也要跟着转：转 90 度之后客户机的「上」朝向屏幕右边，
// 玩家按的「右」才是客户机的「上」，所以按键要往反方向转回去。
// 非方向键原样返回。
static int32_t rotateKey(int32_t key) {
    static const int32_t dirs[4] = {MR_KEY_UP, MR_KEY_RIGHT, MR_KEY_DOWN, MR_KEY_LEFT};
    if (screenRotate == 0) {
        return key;
    }
    for (int i = 0; i < 4; i++) {
        if (dirs[i] == key) {
            return dirs[(i + 4 - screenRotate / 90) % 4];
        }
    }
    return key;
}

static void sendKey(int16 type, int32_t key) {
    event(type, rotateKey(key), 0);
}

// http://wiki.libsdl.org/Tutorials
// http://lazyfoo.net/tutorials/SDL/index.php

static SDL_TimerID timeId = 0;
static SDL_Window *window;
static bool isMouseDown = false;
static bool isEditMode = false;
static int32_t editMaxSize = 0;
static char *holdEditText = NULL;

static SDL_Keycode isKeyDown = SDLK_UNKNOWN;

// 掌机上的实体按键是一个手柄设备(ANBERNIC-keys)，不会产生键盘事件，
// 需要走 SDL_GameController 那一套。映射关系见 controllerEvent()。
static SDL_GameController *controller = NULL;
static int isBtnDown = -1;

// L2/R2 在 gamecontrollerdb 里被登记成 trigger，SDL 按轴上报，
// 这里折算成两个虚拟按键，方便和其它按键走同一套按下/弹起逻辑。
#define VBTN_TRIGGERLEFT (SDL_CONTROLLER_BUTTON_MAX + 0)
#define VBTN_TRIGGERRIGHT (SDL_CONTROLLER_BUTTON_MAX + 1)
#define TRIGGER_THRESHOLD 16000

void saveEditText(char *str) {
    uint8_t *utf8Str = (uint8_t *)str;
    int32_t n = 0;
    while (*utf8Str && (n < editMaxSize)) {
        if (*utf8Str < 0x80) {  // 1 Byte
            utf8Str += 1;
        } else if ((*utf8Str & 0xe0) == 0xc0) {  // 2 Bytes
            utf8Str += 2;
        } else if ((*utf8Str & 0xf0) == 0xe0) {  // 3 Bytes
            utf8Str += 3;
        } else {
            break;
        }
        n++;
    }
    if (holdEditText != NULL) {
        my_freeExt(holdEditText);
        holdEditText = NULL;
    }
    uint32_t len = (uint32_t)utf8Str - (uint32_t)str;
    holdEditText = my_mallocExt(len + 1);
    memcpy(holdEditText, str, len);
    holdEditText[len] = '\0';
}

int32_t editCreate(const char *title, const char *text, int32_t type, int32_t max_size) {
    isEditMode = true;
    editMaxSize = max_size;
    SDL_Log("title: '%s', text: '%s', type: %d, max_size: %d", title, text, type, max_size);
    if (SDL_SetClipboardText(text) == 0) {
        SDL_Log("编辑内容已复制到剪贴板，按ctrl+v输入内容，按ctrl+z取消");
    } else {
        SDL_Log("无法使用剪贴板");
    }
    return 1234;
}

int32 editRelease(int32 edit) {
    isEditMode = false;
    if (holdEditText != NULL) {
        my_freeExt(holdEditText);
        holdEditText = NULL;
    }
    return MR_SUCCESS;
}

char *editGetText(int32 edit) {
    SDL_Log("editGetText(): '%s'", holdEditText);
    return holdEditText;
}

void guiDrawBitmap(uint16_t *bmp, int32_t x, int32_t y, int32_t w, int32_t h) {
    SDL_Surface *surface = SDL_GetWindowSurface(window);
    if (surface == NULL) {
        printf("SDL_GetWindowSurface err: %s\n", SDL_GetError());
        return;
    }
    // 窗口尺寸变了(含第一次绘制)要重算 viewport，并把画面外的黑边刷一遍，
    // 不然会残留上一次的内容
    if (updateViewport(surface)) {
        SDL_FillRect(surface, NULL, SDL_MapRGB(surface->format, 0, 0, 0));
    }
    if (SDL_MUSTLOCK(surface)) {
        if (SDL_LockSurface(surface) != 0) printf("SDL_LockSurface err\n");
    }
    for (int32_t j = 0; j < h; j++) {
        for (int32_t i = 0; i < w; i++) {
            int32_t xx = x + i;
            int32_t yy = y + j;
            if (xx < 0 || yy < 0 || xx >= SCREEN_WIDTH || yy >= SCREEN_HEIGHT) {
                continue;
            }
            uint16_t color = *(bmp + (xx + yy * SCREEN_WIDTH));
            Uint32 pixel = SDL_MapRGB(surface->format, PIXEL565R(color), PIXEL565G(color), PIXEL565B(color));
            int32_t wx, wy;
            viewMap(xx, yy, &wx, &wy);
            // 放大之后一个客户机像素对应窗口上 viewScale x viewScale 个像素
            for (int32_t m = 0; m < viewScale; m++) {
                if (wy + m < 0 || wy + m >= surface->h) continue;
                Uint32 *p = (Uint32 *)(((Uint8 *)surface->pixels) + surface->pitch * (wy + m));
                for (int32_t n = 0; n < viewScale; n++) {
                    if (wx + n < 0 || wx + n >= surface->w) continue;
                    p[wx + n] = pixel;
                }
            }
        }
    }
    if (SDL_MUSTLOCK(surface)) SDL_UnlockSurface(surface);
    if (SDL_UpdateWindowSurface(window) != 0)
        printf("SDL_UpdateWindowSurface err\n");
}

#ifdef __EMSCRIPTEN__
EMSCRIPTEN_KEEPALIVE
void setEventEnable(int v) {
    int state = v ? SDL_ENABLE : SDL_DISABLE;
    SDL_EventState(SDL_TEXTINPUT, state);
    SDL_EventState(SDL_KEYDOWN, state);
    SDL_EventState(SDL_KEYUP, state);
    SDL_EventState(SDL_MOUSEMOTION, state);
    SDL_EventState(SDL_MOUSEBUTTONDOWN, state);
    SDL_EventState(SDL_MOUSEBUTTONUP, state);
}
#endif

// SDL_AddTimer() 的回调跑在独立的定时器线程上，直接在里面调 timer() 的话，
// mrp 的绘制(guiDrawBitmap -> SDL_GetWindowSurface/SDL_UpdateWindowSurface)
// 就发生在非主线程。掌机的 sunxi/mali 后端走 EGL，EGL 上下文是绑定线程的，
// 从别的线程提交会卡住；主线程这时又在 bridge 的 mutex 上等，整个程序就死住了
// (表现为能进列表、一进游戏就不动)。这里只投递一个自定义事件，
// 真正的 mr_timer() 回到主线程的事件循环里执行。
static Uint32 timerEventType = (Uint32)-1;

uint32_t timerCb(uint32_t interval, void *param) {
    SDL_RemoveTimer(timeId);
    timeId = 0;
    if (timerEventType != (Uint32)-1) {
        SDL_Event ev;
        SDL_zero(ev);
        ev.type = timerEventType;
        SDL_PushEvent(&ev);
    }
    return 0;
}

int32_t timerStart(uint16_t t) {
    if (!timeId) {
        timeId = SDL_AddTimer(t, timerCb, NULL);
    } else {
        SDL_RemoveTimer(timeId);
        timeId = SDL_AddTimer(t, timerCb, NULL);
    }
    return MR_SUCCESS;
}

int32_t timerStop() {
    if (timeId) {
        SDL_RemoveTimer(timeId);
        timeId = 0;
    }
    return MR_SUCCESS;
}

static void keyEvent(int16 type, SDL_Keycode code) {
    if (code >= SDLK_0 && code <= SDLK_9) {
        int32_t key = MR_KEY_0 + (code - SDLK_0);
        event(type, key, 0);  // 按键 0-9
        return;
    }
    switch (code) {
        case SDLK_KP_0:
            sendKey(type, MR_KEY_0);
            break;
        case SDLK_KP_1:
            sendKey(type, MR_KEY_1);
            break;
        case SDLK_KP_2:
            sendKey(type, MR_KEY_2);
            break;
        case SDLK_KP_3:
            sendKey(type, MR_KEY_3);
            break;
        case SDLK_KP_4:
            sendKey(type, MR_KEY_4);
            break;
        case SDLK_KP_5:
            sendKey(type, MR_KEY_5);
            break;
        case SDLK_KP_6:
            sendKey(type, MR_KEY_6);
            break;
        case SDLK_KP_7:
            sendKey(type, MR_KEY_7);
            break;
        case SDLK_KP_8:
            sendKey(type, MR_KEY_8);
            break;
        case SDLK_KP_9:
            sendKey(type, MR_KEY_9);
            break;
        case SDLK_KP_ENTER:
        case SDLK_RETURN:                   // 回车键
            sendKey(type, MR_KEY_SELECT);   // 确认/选择/ok
            break;
        case SDLK_EQUALS:                  // 等号
            sendKey(type, MR_KEY_POUND);   // 按键 #
            break;
        case SDLK_MINUS:                  // 减号
            sendKey(type, MR_KEY_STAR);   // 按键 *
            break;
        case SDLK_w:
        case SDLK_UP:  // 上
            sendKey(type, MR_KEY_UP);
            break;
        case SDLK_s:
        case SDLK_DOWN:  // 下
            sendKey(type, MR_KEY_DOWN);
            break;
        case SDLK_a:
        case SDLK_LEFT:  // 左
            sendKey(type, MR_KEY_LEFT);
            break;
        case SDLK_d:
        case SDLK_RIGHT:  // 右
            sendKey(type, MR_KEY_RIGHT);
            break;
        case SDLK_q:
        case SDLK_LEFTBRACKET:                // 左中括号
            sendKey(type, MR_KEY_SOFTLEFT);   // 左功能键
            break;
        case SDLK_e:
        case SDLK_RIGHTBRACKET:                // 右中括号
            sendKey(type, MR_KEY_SOFTRIGHT);   // 右功能键
            break;
        case SDLK_TAB:
            sendKey(type, MR_KEY_SEND);  // 接听键
            break;
        case SDLK_ESCAPE:
            sendKey(type, MR_KEY_POWER);  // 挂机键
            break;
        default:
            printf("key:%d\n", code);
            break;
    }
}

// 手柄按键映射。按键号取自 resource/gamecontrollerdb.txt 中的 ANBERNIC-keys 条目：
//   dpup:h0.1,dpdown:h0.4,dpleft:h0.8,dpright:h0.2,
//   a:b0,b:b1,y:b2,x:b3,leftshoulder:b4,rightshoulder:b5,
//   back:b6,start:b7,guide:b8,lefttrigger:b9,righttrigger:b10
// 这条记录已经内置进 SDL2(见 resource/patch/SDL2-2.32.10.patch)，所以这里
// 直接用 SDL_CONTROLLER_BUTTON_* 而不用自己解析 js 的按键号。
// 实体位置按 resource/gamepad_test.c 的布局约定：A=右 B=下 X=上 Y=左。
static void controllerEvent(int16 type, int btn) {
    switch (btn) {
        case SDL_CONTROLLER_BUTTON_DPAD_UP:
            sendKey(type, MR_KEY_UP);
            break;
        case SDL_CONTROLLER_BUTTON_DPAD_DOWN:
            sendKey(type, MR_KEY_DOWN);
            break;
        case SDL_CONTROLLER_BUTTON_DPAD_LEFT:
            sendKey(type, MR_KEY_LEFT);
            break;
        case SDL_CONTROLLER_BUTTON_DPAD_RIGHT:
            sendKey(type, MR_KEY_RIGHT);
            break;
        case SDL_CONTROLLER_BUTTON_A:  // 右键：确认
            sendKey(type, MR_KEY_SELECT);
            break;
        case SDL_CONTROLLER_BUTTON_B:  // 下键：右软键(返回)
            sendKey(type, MR_KEY_SOFTRIGHT);
            break;
        case SDL_CONTROLLER_BUTTON_X:  // 上键：左软键(选项)
            sendKey(type, MR_KEY_SOFTLEFT);
            break;
        case SDL_CONTROLLER_BUTTON_Y:  // 左键：不少 mrp 游戏用 5 当确认/开火
            sendKey(type, MR_KEY_5);
            break;
        case SDL_CONTROLLER_BUTTON_LEFTSHOULDER:
            sendKey(type, MR_KEY_STAR);  // *
            break;
        case SDL_CONTROLLER_BUTTON_RIGHTSHOULDER:
            sendKey(type, MR_KEY_POUND);  // #
            break;
        case VBTN_TRIGGERLEFT:
            sendKey(type, MR_KEY_1);
            break;
        case VBTN_TRIGGERRIGHT:
            sendKey(type, MR_KEY_3);
            break;
        case SDL_CONTROLLER_BUTTON_BACK:  // SELECT：接听键
            sendKey(type, MR_KEY_SEND);
            break;
        case SDL_CONTROLLER_BUTTON_START:  // START：挂机键
            sendKey(type, MR_KEY_POWER);
            break;
        case SDL_CONTROLLER_BUTTON_GUIDE: {  // MENU：掌机上没有别的办法关窗口
            if (type == MR_KEY_RELEASE) {
                SDL_Event quit = {.type = SDL_QUIT};
                SDL_PushEvent(&quit);
            }
            break;
        }
        default:
            printf("button:%d\n", btn);
            break;
    }
}

// 一次只上报一个按键，和键盘那边 isKeyDown 的处理保持一致
static void controllerPress(int btn) {
    if (isBtnDown == -1) {
        isBtnDown = btn;
        controllerEvent(MR_KEY_PRESS, btn);
    }
}

static void controllerRelease(int btn) {
    if (isBtnDown == btn) {
        isBtnDown = -1;
        controllerEvent(MR_KEY_RELEASE, btn);
    }
}

void loop() {
    SDL_Event ev;
    bool isLoop = true;

#if defined(__EMSCRIPTEN__)
#else
    while (isLoop)
#endif
    {
#if defined(__EMSCRIPTEN__)
        while (SDL_PollEvent(&ev))
#else
        while (SDL_WaitEvent(&ev))
#endif
        {
            if (ev.type == SDL_QUIT) {
                isLoop = false;
                // emscripten_cancel_main_loop();
                break;
            }
            if (ev.type == timerEventType) {
                timer();
                continue;
            }
            if (isEditMode) {
                switch (ev.type) {
                    case SDL_KEYDOWN: {
                        if (SDL_GetModState() & KMOD_CTRL) {
                            if (ev.key.keysym.sym == SDLK_z) {  // 取消编辑框输入
                                // MR_DIALOG_KEY_CANCEL=1
                                event(MR_DIALOG_EVENT, 1, 0);
                                SDL_Log("取消输入");
                                continue;
                            } else if (ev.key.keysym.sym == SDLK_v) {  // 编辑框输入
                                char *str = SDL_GetClipboardText();
                                saveEditText(str);
                                SDL_free(str);
                                // MR_DIALOG_KEY_OK=0
                                event(MR_DIALOG_EVENT, 0, 0);
                                continue;
                            }
                        }
                    }
                    case SDL_MOUSEBUTTONDOWN:
                        SDL_Log("ctrl+v输入内容，ctrl+z取消输入");
                        break;
                    case SDL_CONTROLLERBUTTONDOWN:
                        // 掌机上没有键盘也没有剪贴板，至少要能把编辑框关掉
                        if (ev.cbutton.button == SDL_CONTROLLER_BUTTON_B) {
                            // MR_DIALOG_KEY_CANCEL=1
                            event(MR_DIALOG_EVENT, 1, 0);
                            SDL_Log("取消输入");
                        }
                        break;
                }
                continue;
            }
            switch (ev.type) {
                case SDL_CONTROLLERDEVICEADDED:
                    if (controller == NULL) {
                        controller = SDL_GameControllerOpen(ev.cdevice.which);
                        if (controller) {
                            SDL_Log("手柄已连接: %s", SDL_GameControllerName(controller));
                        } else {
                            SDL_Log("打开手柄失败: %s", SDL_GetError());
                        }
                    }
                    break;
                case SDL_CONTROLLERDEVICEREMOVED:
                    if (controller != NULL &&
                        ev.cdevice.which == SDL_JoystickInstanceID(SDL_GameControllerGetJoystick(controller))) {
                        SDL_GameControllerClose(controller);
                        controller = NULL;
                        isBtnDown = -1;
                        SDL_Log("手柄已断开");
                    }
                    break;
                case SDL_CONTROLLERBUTTONDOWN:
                    controllerPress(ev.cbutton.button);
                    break;
                case SDL_CONTROLLERBUTTONUP:
                    controllerRelease(ev.cbutton.button);
                    break;
                case SDL_CONTROLLERAXISMOTION: {
                    int btn;
                    if (ev.caxis.axis == SDL_CONTROLLER_AXIS_TRIGGERLEFT) {
                        btn = VBTN_TRIGGERLEFT;
                    } else if (ev.caxis.axis == SDL_CONTROLLER_AXIS_TRIGGERRIGHT) {
                        btn = VBTN_TRIGGERRIGHT;
                    } else {
                        break;
                    }
                    if (ev.caxis.value > TRIGGER_THRESHOLD) {
                        controllerPress(btn);
                    } else {
                        controllerRelease(btn);
                    }
                    break;
                }
                case SDL_KEYDOWN:
                    if (isKeyDown == SDLK_UNKNOWN) {
                        isKeyDown = ev.key.keysym.sym;
                        keyEvent(MR_KEY_PRESS, ev.key.keysym.sym);
                    }
                    break;
                case SDL_KEYUP:
                    if (isKeyDown == ev.key.keysym.sym) {
                        isKeyDown = SDLK_UNKNOWN;
                        keyEvent(MR_KEY_RELEASE, ev.key.keysym.sym);
                    }
                    break;
                // 鼠标/触摸落在窗口坐标上，要按当前的旋转和放大倍率换回客户机坐标
                case SDL_MOUSEMOTION: {
                    if (isMouseDown) {
                        int32_t gx, gy;
                        viewUnmap(ev.motion.x, ev.motion.y, &gx, &gy);
                        event(MR_MOUSE_MOVE, gx, gy);
                    }
                    break;
                }
                case SDL_MOUSEBUTTONDOWN: {
                    int32_t gx, gy;
                    viewUnmap(ev.motion.x, ev.motion.y, &gx, &gy);
                    isMouseDown = true;
                    event(MR_MOUSE_DOWN, gx, gy);
                    break;
                }
                case SDL_MOUSEBUTTONUP: {
                    int32_t gx, gy;
                    viewUnmap(ev.motion.x, ev.motion.y, &gx, &gy);
                    isMouseDown = false;
                    event(MR_MOUSE_UP, gx, gy);
                    break;
                }
            }
        }
    }
}

static void usage(const char *prog) {
    printf(
        "用法: %s [选项]\n"
        "  -r, --rotate <角度>  画面顺时针旋转 0/90/180/270 度，默认 0。\n"
        "                       客户机画面是 240x320 的竖屏，掌机屏幕 640x480，\n"
        "                       转 90 或 270 度之后正好整数放大 2 倍铺满屏幕(点对点)。\n"
        "                       方向键会跟着一起旋转。\n"
        "  -h, --help           显示这段帮助\n",
        prog);
}

// 只有 --rotate 一个选项，没必要引入 getopt
static bool parseArgs(int argc, char *args[]) {
    for (int i = 1; i < argc; i++) {
        const char *arg = args[i];
        const char *val = NULL;
        if (strcmp(arg, "-h") == 0 || strcmp(arg, "--help") == 0) {
            usage(args[0]);
            exit(0);
        }
        if (strncmp(arg, "--rotate=", 9) == 0) {
            val = arg + 9;
        } else if (strcmp(arg, "-r") == 0 || strcmp(arg, "--rotate") == 0) {
            if (i + 1 >= argc) {
                printf("%s 缺少参数\n", arg);
                return false;
            }
            val = args[++i];
        } else {
            printf("未知参数: %s\n", arg);
            return false;
        }
        int deg = atoi(val);
        if (deg != 0 && deg != 90 && deg != 180 && deg != 270) {
            printf("--rotate 只支持 0/90/180/270，收到: %s\n", val);
            return false;
        }
        screenRotate = deg;
    }
    return true;
}

int main(int argc, char *args[]) {
    if (!parseArgs(argc, args)) {
        usage(args[0]);
        return -1;
    }

#ifdef __x86_64__
    printf("__x86_64__\n");
#elif __i386__
    printf("__i386__\n");
#endif

    printf("CODE_ADDRESS:0x%X, CODE_SIZE:0x%X\n", CODE_ADDRESS, CODE_SIZE);
    printf("STACK_ADDRESS:0x%X, STACK_SIZE:0x%X\n", STACK_ADDRESS, STACK_SIZE);
    printf("MEMORY_MANAGER_ADDRESS:0x%X, MEMORY_MANAGER_SIZE:0x%X\n", MEMORY_MANAGER_ADDRESS, MEMORY_MANAGER_SIZE);
    printf("START_ADDRESS:0x%X, END_ADDRESS:0x%X\n", START_ADDRESS, END_ADDRESS);
    printf("TOTAL_MEMORY:0x%X(%d)\n", TOTAL_MEMORY, TOTAL_MEMORY);

    if (SDL_Init(SDL_INIT_VIDEO | SDL_INIT_TIMER | SDL_INIT_AUDIO | SDL_INIT_JOYSTICK | SDL_INIT_GAMECONTROLLER) < 0) {
        printf("SDL could not initialize! SDL_Error: %s\n", SDL_GetError());
        return -1;
    }

    timerEventType = SDL_RegisterEvents(1);
    if (timerEventType == (Uint32)-1) {
        printf("SDL_RegisterEvents() fail: %s\n", SDL_GetError());
        return -1;
    }

    // 掌机上 sunxi 后端会忽略这里的尺寸直接开全屏(640x480)，画面居中放大由
    // updateViewport() 处理；桌面调试时按旋转后的尺寸开窗口，正好 1:1
    int winW, winH;
    viewSize(&winW, &winH);
    window = SDL_CreateWindow("vmrp", SDL_WINDOWPOS_CENTERED, SDL_WINDOWPOS_CENTERED, winW, winH, SDL_WINDOW_OPENGL);
    if (window == NULL) {
        printf("Window could not be created! SDL_Error: %s\n", SDL_GetError());
        return -1;
    }

#ifdef VMRP_HOME
    // cfunction.ext 和 mythroad/ 都是按当前目录查找的，从别处启动时自动切过去，
    // 不然只能靠外面的 wrapper 脚本先 cd，直接跑 vmrp 就会 loadCode fail
    if (access("cfunction.ext", R_OK) != 0 && chdir(VMRP_HOME) != 0) {
        printf("chdir('%s') fail: %s\n", VMRP_HOME, strerror(errno));
    }
#endif

#ifndef __EMSCRIPTEN__
    // wasm 版的声音由 JS 那边负责(js_playSound)，只有原生版要自己开音频设备。
    // 打不开只是没声音，不影响模拟器跑，所以不检查返回值
    soundInit();
#endif

    // 上游忽略了这里的返回值，初始化失败后照样进事件循环，
    // 结果第一次按键就会从 PC=0 取指，报成 UC_ERR_FETCH_UNMAPPED
    if (startVmrp() == MR_FAILED) {
        printf("startVmrp() fail: 当前目录下找不到 cfunction.ext 或 mythroad/\n");
        return -1;
    }

#if defined(__EMSCRIPTEN__)
    emscripten_set_main_loop(loop, 0, 1);
#else
    loop();
    soundDeinit();
#endif
    return 0;
}
