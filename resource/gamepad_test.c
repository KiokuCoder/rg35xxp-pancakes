#include <SDL2/SDL.h>
#include <SDL2/SDL_ttf.h>
#include <stdbool.h>
#include <stdio.h>

// 窗口分辨率
#define SCREEN_WIDTH 640
#define SCREEN_HEIGHT 480

// 定义我们要测试的按键枚举
enum {
    BTN_UP, BTN_DOWN, BTN_LEFT, BTN_RIGHT,
    BTN_A, BTN_B, BTN_X, BTN_Y,
    BTN_L1, BTN_R1, BTN_L2, BTN_R2,
    BTN_SELECT, BTN_START, BTN_MENU,
    BTN_COUNT
};

// UI 按键结构体
typedef struct {
    SDL_Rect rect;
    const char* label;
    bool pressed;
} ButtonUI;

ButtonUI buttons[BTN_COUNT];

// 初始化按钮布局坐标
void InitButtons() {
    // 方向键 (左侧) - 十字排布 中心点约为 (120, 240)
    buttons[BTN_UP]    = (ButtonUI){{100, 160, 40, 40}, "UP", false};
    buttons[BTN_DOWN]  = (ButtonUI){{100, 240, 40, 40}, "DOWN", false};
    buttons[BTN_LEFT]  = (ButtonUI){{60,  200, 40, 40}, "LEFT", false};
    buttons[BTN_RIGHT] = (ButtonUI){{140, 200, 40, 40}, "RIGHT", false};

    // 动作键 (右侧) - 菱形排布 中心点约为 (520, 240) (采用Xbox/常用开源掌机标准布局)
    buttons[BTN_X] = (ButtonUI){{500, 160, 40, 40}, "X", false};
    buttons[BTN_B] = (ButtonUI){{500, 240, 40, 40}, "B", false};
    buttons[BTN_Y] = (ButtonUI){{460, 200, 40, 40}, "Y", false};
    buttons[BTN_A] = (ButtonUI){{540, 200, 40, 40}, "A", false};

    // 肩键和扳机键
    buttons[BTN_L1] = (ButtonUI){{20,  60, 80, 30}, "L1", false};
    buttons[BTN_L2] = (ButtonUI){{20,  20, 80, 30}, "L2", false};
    buttons[BTN_R1] = (ButtonUI){{540, 60, 80, 30}, "R1", false};
    buttons[BTN_R2] = (ButtonUI){{540, 20, 80, 30}, "R2", false};

    // 系统键 (底部中央)
    buttons[BTN_SELECT] = (ButtonUI){{220, 400, 60, 30}, "SELECT", false};
    buttons[BTN_MENU]   = (ButtonUI){{290, 400, 60, 30}, "MENU", false};
    buttons[BTN_START]  = (ButtonUI){{360, 400, 60, 30}, "START", false};
}

// 渲染居中文字的辅助函数
void DrawTextCenter(SDL_Renderer* renderer, TTF_Font* font, const char* text, SDL_Rect rect) {
    SDL_Color color = {255, 255, 255, 255}; // 白色文字
    SDL_Surface* surface = TTF_RenderText_Blended(font, text, color);
    if (!surface) return;
    
    SDL_Texture* texture = SDL_CreateTextureFromSurface(renderer, surface);
    if (!texture) {
        SDL_FreeSurface(surface);
        return;
    }

    // 计算居中坐标
    SDL_Rect textRect;
    textRect.w = surface->w;
    textRect.h = surface->h;
    textRect.x = rect.x + (rect.w - textRect.w) / 2;
    textRect.y = rect.y + (rect.h - textRect.h) / 2;

    SDL_RenderCopy(renderer, texture, NULL, &textRect);
    
    SDL_DestroyTexture(texture);
    SDL_FreeSurface(surface);
}

// 处理手柄按键事件
void HandleControllerButton(Uint8 button, bool pressed) {
    switch (button) {
        case SDL_CONTROLLER_BUTTON_DPAD_UP:    buttons[BTN_UP].pressed = pressed; break;
        case SDL_CONTROLLER_BUTTON_DPAD_DOWN:  buttons[BTN_DOWN].pressed = pressed; break;
        case SDL_CONTROLLER_BUTTON_DPAD_LEFT:  buttons[BTN_LEFT].pressed = pressed; break;
        case SDL_CONTROLLER_BUTTON_DPAD_RIGHT: buttons[BTN_RIGHT].pressed = pressed; break;
        case SDL_CONTROLLER_BUTTON_A:          buttons[BTN_A].pressed = pressed; break;
        case SDL_CONTROLLER_BUTTON_B:          buttons[BTN_B].pressed = pressed; break;
        case SDL_CONTROLLER_BUTTON_X:          buttons[BTN_X].pressed = pressed; break;
        case SDL_CONTROLLER_BUTTON_Y:          buttons[BTN_Y].pressed = pressed; break;
        case SDL_CONTROLLER_BUTTON_LEFTSHOULDER:  buttons[BTN_L1].pressed = pressed; break;
        case SDL_CONTROLLER_BUTTON_RIGHTSHOULDER: buttons[BTN_R1].pressed = pressed; break;
        case SDL_CONTROLLER_BUTTON_BACK:       buttons[BTN_SELECT].pressed = pressed; break;
        case SDL_CONTROLLER_BUTTON_START:      buttons[BTN_START].pressed = pressed; break;
        case SDL_CONTROLLER_BUTTON_GUIDE:      buttons[BTN_MENU].pressed = pressed; break; // Menu键通常映射为Guide
    }
}

// 处理 PC 键盘测试回退 (为了方便在PC上测试)
void HandleKeyboard(SDL_Keycode key, bool pressed) {
    switch (key) {
        case SDLK_UP:    buttons[BTN_UP].pressed = pressed; break;
        case SDLK_DOWN:  buttons[BTN_DOWN].pressed = pressed; break;
        case SDLK_LEFT:  buttons[BTN_LEFT].pressed = pressed; break;
        case SDLK_RIGHT: buttons[BTN_RIGHT].pressed = pressed; break;
        case SDLK_k:     buttons[BTN_A].pressed = pressed; break;
        case SDLK_l:     buttons[BTN_B].pressed = pressed; break;
        case SDLK_i:     buttons[BTN_X].pressed = pressed; break;
        case SDLK_j:     buttons[BTN_Y].pressed = pressed; break;
        case SDLK_q:     buttons[BTN_L1].pressed = pressed; break;
        case SDLK_1:     buttons[BTN_L2].pressed = pressed; break;
        case SDLK_e:     buttons[BTN_R1].pressed = pressed; break;
        case SDLK_3:     buttons[BTN_R2].pressed = pressed; break;
        case SDLK_SPACE: buttons[BTN_SELECT].pressed = pressed; break;
        case SDLK_RETURN:buttons[BTN_START].pressed = pressed; break;
        case SDLK_ESCAPE:buttons[BTN_MENU].pressed = pressed; break;
    }
}

int main(int argc, char* args[]) {
    // 初始化 SDL2 (包含视频和游戏手柄子系统)
    if (SDL_Init(SDL_INIT_VIDEO | SDL_INIT_JOYSTICK | SDL_INIT_GAMECONTROLLER) < 0) {
        printf("SDL Init failed: %s\n", SDL_GetError());
        return -1;
    }

    // 初始化 TTF
    if (TTF_Init() == -1) {
        printf("TTF Init failed: %s\n", TTF_GetError());
        return -1;
    }

    // 打印手柄信息
    int num_joysticks = SDL_NumJoysticks();
    printf("检测到 %d 个手柄。\n\n", num_joysticks);

    for (int i = 0; i < num_joysticks; ++i) {
        // 3. 获取手柄名称
        const char* name = SDL_JoystickNameForIndex(i);
        
        // 4. 获取 GUID 结构体 (无需打开手柄)
        SDL_JoystickGUID guid = SDL_JoystickGetDeviceGUID(i);
        
        // 5. 将 GUID 转换为字符串
        // GUID 字符串长度固定为 32 个字符 + 1 个结束符 '\0'，所以缓冲区至少需要 33 字节
        char guid_str[64]; 
        SDL_JoystickGetGUIDString(guid, guid_str, sizeof(guid_str));

        // 打印结果
        printf("手柄索引: %d\n", i);
        printf("手柄名称: %s\n", name ? name : "Unknown");
        printf("手柄 GUID: %s\n", guid_str);
        printf("---------------------------\n");
    }

    SDL_Window* window = SDL_CreateWindow("Gamepad Tester", 
                                          SDL_WINDOWPOS_CENTERED, SDL_WINDOWPOS_CENTERED, 
                                          SCREEN_WIDTH, SCREEN_HEIGHT, SDL_WINDOW_SHOWN);
    SDL_Renderer* renderer = SDL_CreateRenderer(window, -1, SDL_RENDERER_ACCELERATED | SDL_RENDERER_PRESENTVSYNC);

    // 加载字体 (请确保同级目录下有 font.ttf 文件)
    TTF_Font* font = TTF_OpenFont("font.ttf", 16);
    if (!font) {
        printf("Failed to load font.ttf! Error: %s\n", TTF_GetError());
        printf("Please place a 'font.ttf' file in the executable directory.\n");
        // 即使没有字体，也继续运行，只是没有文字
    }

    InitButtons();

    SDL_GameController* controller = NULL;
    bool quit = false;
    SDL_Event e;

    while (!quit) {
        while (SDL_PollEvent(&e) != 0) {
            if (e.type == SDL_QUIT) {
                quit = true;
            } 
            // 键盘事件 (用于 PC 端模拟测试)
            else if (e.type == SDL_KEYDOWN || e.type == SDL_KEYUP) {
                HandleKeyboard(e.key.keysym.sym, e.type == SDL_KEYDOWN);
            }
            // 手柄热插拔事件
            else if (e.type == SDL_CONTROLLERDEVICEADDED) {
                if (!controller) {
                    controller = SDL_GameControllerOpen(e.cdevice.which);
                    printf("Controller connected: %s\n", SDL_GameControllerName(controller));
                }
            } else if (e.type == SDL_CONTROLLERDEVICEREMOVED) {
                if (controller && e.cdevice.which == SDL_JoystickInstanceID(SDL_GameControllerGetJoystick(controller))) {
                    SDL_GameControllerClose(controller);
                    controller = NULL;
                    printf("Controller disconnected.\n");
                }
            }
            // 手柄按键事件
            else if (e.type == SDL_CONTROLLERBUTTONDOWN || e.type == SDL_CONTROLLERBUTTONUP) {
                HandleControllerButton(e.cbutton.button, e.type == SDL_CONTROLLERBUTTONDOWN);
            }
            // 手柄扳机键(L2/R2)通常被识别为轴(Axis)
            else if (e.type == SDL_CONTROLLERAXISMOTION) {
                if (e.caxis.axis == SDL_CONTROLLER_AXIS_TRIGGERLEFT) {
                    buttons[BTN_L2].pressed = (e.caxis.value > 16000); // 设定一个阈值
                } else if (e.caxis.axis == SDL_CONTROLLER_AXIS_TRIGGERRIGHT) {
                    buttons[BTN_R2].pressed = (e.caxis.value > 16000);
                }
            }
        }

        // --- 渲染部分 ---
        
        // 1. 清空屏幕 (深灰色背景)
        SDL_SetRenderDrawColor(renderer, 30, 30, 40, 255);
        SDL_RenderClear(renderer);

        // 2. 绘制所有按键
        for (int i = 0; i < BTN_COUNT; i++) {
            ButtonUI* btn = &buttons[i];

            // 绘制按键填充色 (按下为绿色，松开为浅灰)
            if (btn->pressed) {
                SDL_SetRenderDrawColor(renderer, 50, 205, 50, 255); // 绿色
            } else {
                SDL_SetRenderDrawColor(renderer, 80, 80, 80, 255);  // 灰色
            }
            SDL_RenderFillRect(renderer, &btn->rect);

            // 绘制按键边框 (白色)
            SDL_SetRenderDrawColor(renderer, 255, 255, 255, 255);
            SDL_RenderDrawRect(renderer, &btn->rect);

            // 绘制文字
            if (font) {
                DrawTextCenter(renderer, font, btn->label, btn->rect);
            }
        }

        // 3. 更新屏幕
        SDL_RenderPresent(renderer);
    }

    // 清理资源
    if (font) TTF_CloseFont(font);
    if (controller) SDL_GameControllerClose(controller);
    SDL_DestroyRenderer(renderer);
    SDL_DestroyWindow(window);
    TTF_Quit();
    SDL_Quit();

    return 0;
}
