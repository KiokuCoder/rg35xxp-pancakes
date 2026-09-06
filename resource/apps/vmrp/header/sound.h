#ifndef __VMRP_SOUND_H__
#define __VMRP_SOUND_H__

#include "types.h"

// 声音类型，取值和 mrporting.h / bridge.c 里的 MR_SOUND_TYPE 一一对应。
// 换个名字是因为 bridge.c 自己已经声明了一份同名的匿名枚举。
enum {
    SOUND_MIDI = 0,
    SOUND_WAV,
    SOUND_MP3,
    SOUND_AMR,
    SOUND_PCM,  // 8K 16bit 单声道裸数据
    SOUND_TYPE_COUNT
};

// 打开音频设备，失败只是没声音，不影响模拟器继续跑
int32 soundInit(void);
void soundDeinit(void);

// data 指向客户机内存，函数返回后就不保证有效了，所以内部一律先复制/解码出来
int32 soundPlay(int32 type, const void *data, uint32 dataLen, int32 loop);
int32 soundStop(int32 type);

#endif
