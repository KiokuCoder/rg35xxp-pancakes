// 声音后端。上游只在 wasm 版实现了声音(mr_playSound 转给 JS 的 js_playSound)，
// windows/linux 原生版是空实现，直接 return MR_SUCCESS，掌机上一点声音都没有。
// 这里按 mrp 的四种声音类型各接一个解码器，统一转成设备格式后混音输出：
//   MIDI -> libADLMIDI，OPL3(YMF262) FM 合成，音色表编译在库里，不需要额外的音色库文件
//   MP3  -> libmpg123
//   WAV  -> SDL 自带的 SDL_LoadWAV_RW()
//   PCM  -> 8K 16bit 单声道裸数据，只做重采样
//   AMR  -> 没有解码器，返回 MR_FAILED
#include "./header/sound.h"

#include <SDL2/SDL.h>
#include <adlmidi.h>
#include <mpg123.h>
#include <stdbool.h>
#include <stdio.h>
#include <string.h>

// 设备格式。libADLMIDI 固定输出 16bit 立体声，设备开成一样的格式，
// 合成出来的数据就能直接混进 stream，不用每次回调再转一遍。
#define DEV_FREQ 44100
#define DEV_FORMAT AUDIO_S16SYS
#define DEV_CHANNELS 2
#define DEV_SAMPLES 1024  // 每次回调的帧数，44100Hz 下约 23ms

// mrp 的 PCM 类型，见 mrporting.h 中 MR_SOUND_TYPE 的注释
#define MRP_PCM_FREQ 8000
#define MRP_PCM_FORMAT AUDIO_S16LSB
#define MRP_PCM_CHANNELS 1

// 解码结果的上限，坏数据不至于把内存吃光
#define PCM_MAX_LEN (32 * 1024 * 1024)
#define PCM_STEP_LEN (128 * 1024)

// 一路已经解码好的声音。mr_stopSound() 是按类型停的，所以每种类型各占一路，
// 同时响的互不影响(典型用法是 MIDI 放背景音乐，WAV 放音效)。
typedef struct {
    Uint8 *buf;  // 设备格式的 PCM，NULL 表示这一路没在播
    Uint32 len;
    Uint32 pos;
    bool loop;
} Voice;

static SDL_AudioDeviceID dev = 0;
static SDL_AudioSpec devSpec;

// SOUND_MIDI 那一路是实时合成的，不走 voices[]
static Voice voices[SOUND_TYPE_COUNT];

static struct ADL_MIDIPlayer *midi = NULL;
static bool midiPlaying = false;
static short *midiBuf = NULL;  // 回调里给 adl_play() 用的临时缓冲，devSpec.size 字节
static bool mpg123Ready = false;

//////////////////////////////////////////////////////////////////////////////////////////

// 在音频线程上跑，voices[]/midiPlaying 的改动都要先 SDL_LockAudioDevice()
static void mixVoice(Voice *v, Uint8 *stream, int len) {
    int off = 0;
    while (off < len) {
        Uint32 n = v->len - v->pos;
        if (n > (Uint32)(len - off)) {
            n = (Uint32)(len - off);
        }
        SDL_MixAudioFormat(stream + off, v->buf + v->pos, DEV_FORMAT, n, SDL_MIX_MAXVOLUME);
        v->pos += n;
        off += (int)n;
        if (v->pos < v->len) {
            continue;
        }
        if (!v->loop) {
            SDL_free(v->buf);
            v->buf = NULL;
            v->len = 0;
            v->pos = 0;
            return;
        }
        v->pos = 0;  // 循环播放，接着从头填满这次回调
    }
}

static void audioCb(void *userdata, Uint8 *stream, int len) {
    (void)userdata;
    SDL_memset(stream, 0, (size_t)len);

    if (midiPlaying) {
        // adl_play() 要的是采样数不是帧数，一帧两个采样，正好是 len/sizeof(short)
        int samples = len / (int)sizeof(short);
        if (samples > (int)(devSpec.size / sizeof(short))) {
            samples = (int)(devSpec.size / sizeof(short));
        }
        int got = adl_play(midi, samples, midiBuf);
        if (got > 0) {
            SDL_MixAudioFormat(stream, (const Uint8 *)midiBuf, DEV_FORMAT,
                               (Uint32)got * sizeof(short), SDL_MIX_MAXVOLUME);
        }
        if (got < samples) {
            midiPlaying = false;  // 放完了。循环播放时 adl_play() 不会返回不足的采样数
        }
    }

    for (int i = 0; i < SOUND_TYPE_COUNT; i++) {
        if (voices[i].buf != NULL) {
            mixVoice(&voices[i], stream, len);
        }
    }
}

//////////////////////////////////////////////////////////////////////////////////////////

// 把任意格式的 PCM 转成设备格式，返回的缓冲由调用方接管(SDL_free)
static Uint8 *toDevFormat(const Uint8 *src, Uint32 srcLen, SDL_AudioFormat fmt, int channels, int freq, Uint32 *outLen) {
    if (channels <= 0 || freq <= 0) {
        return NULL;
    }
    SDL_AudioStream *stream = SDL_NewAudioStream(fmt, (Uint8)channels, freq, DEV_FORMAT, DEV_CHANNELS, DEV_FREQ);
    if (stream == NULL) {
        printf("SDL_NewAudioStream() fail: %s\n", SDL_GetError());
        return NULL;
    }
    Uint8 *out = NULL;
    if (SDL_AudioStreamPut(stream, src, (int)srcLen) == 0 && SDL_AudioStreamFlush(stream) == 0) {
        int avail = SDL_AudioStreamAvailable(stream);
        if (avail > 0) {
            out = SDL_malloc((size_t)avail);
            if (out != NULL) {
                int got = SDL_AudioStreamGet(stream, out, avail);
                if (got > 0) {
                    *outLen = (Uint32)got;
                } else {
                    SDL_free(out);
                    out = NULL;
                }
            }
        }
    }
    if (out == NULL) {
        printf("音频重采样失败: %s\n", SDL_GetError());
    }
    SDL_FreeAudioStream(stream);
    return out;
}

// 把整段 MP3 解成 16bit PCM，采样率和声道数由 freq/channels 带出来
static Uint8 *decodeMp3(const void *data, uint32 dataLen, Uint32 *outLen, int *freq, int *channels) {
    if (!mpg123Ready) {
        return NULL;
    }
    int err = MPG123_OK;
    mpg123_handle *mh = mpg123_new(NULL, &err);
    if (mh == NULL) {
        printf("mpg123_new() fail: %s\n", mpg123_plain_strerror(err));
        return NULL;
    }

    // 只要 16bit 输出，采样率和声道数保持文件本身的，后面统一交给 toDevFormat()
    mpg123_param(mh, MPG123_ADD_FLAGS, MPG123_QUIET, 0.0);
    mpg123_format_none(mh);
    const long *rates = NULL;
    size_t rateCount = 0;
    mpg123_rates(&rates, &rateCount);
    for (size_t i = 0; i < rateCount; i++) {
        mpg123_format(mh, rates[i], MPG123_MONO | MPG123_STEREO, MPG123_ENC_SIGNED_16);
    }

    Uint8 *out = NULL;
    Uint32 used = 0, cap = 0;
    if (mpg123_open_feed(mh) != MPG123_OK || mpg123_feed(mh, data, dataLen) != MPG123_OK) {
        printf("mpg123 feed fail: %s\n", mpg123_strerror(mh));
        goto end;
    }
    for (;;) {
        if (used == cap) {
            Uint32 next = cap == 0 ? PCM_STEP_LEN : cap * 2;
            if (next > PCM_MAX_LEN) {
                printf("mp3 解码结果超过 %d 字节，截断\n", PCM_MAX_LEN);
                break;
            }
            Uint8 *p = SDL_realloc(out, next);
            if (p == NULL) {
                SDL_free(out);
                out = NULL;
                goto end;
            }
            out = p;
            cap = next;
        }
        size_t done = 0;
        int r = mpg123_read(mh, out + used, cap - used, &done);
        used += (Uint32)done;
        if (r == MPG123_NEW_FORMAT) {
            long rate = 0;
            int ch = 0, enc = 0;
            mpg123_getformat(mh, &rate, &ch, &enc);
            *freq = (int)rate;
            *channels = ch;
            continue;
        }
        if (r == MPG123_ERR) {
            printf("mpg123_read() fail: %s\n", mpg123_strerror(mh));
            break;
        }
        if (r != MPG123_OK) {
            break;  // MPG123_NEED_MORE / MPG123_DONE，数据一次喂完了，到这就是放完了
        }
    }
    if (used == 0) {
        SDL_free(out);
        out = NULL;
    } else {
        *outLen = used;
    }
end:
    mpg123_delete(mh);
    return out;
}

// 接管 buf，把它挂到对应的那一路上，替换掉这一路原来在放的声音
static int32 startVoice(int32 type, Uint8 *buf, Uint32 len, int32 loop) {
    if (buf == NULL || len == 0) {
        SDL_free(buf);
        return MR_FAILED;
    }
    SDL_LockAudioDevice(dev);
    SDL_free(voices[type].buf);
    voices[type].buf = buf;
    voices[type].len = len;
    voices[type].pos = 0;
    voices[type].loop = loop != 0;
    SDL_UnlockAudioDevice(dev);
    return MR_SUCCESS;
}

static int32 playMidi(const void *data, uint32 dataLen, int32 loop) {
    if (midi == NULL) {
        return MR_FAILED;
    }
    SDL_LockAudioDevice(dev);
    midiPlaying = false;
    adl_setLoopEnabled(midi, loop != 0);
    adl_setLoopCount(midi, loop != 0 ? -1 : 0);
    int32 ret = MR_SUCCESS;
    if (adl_openData(midi, data, dataLen) < 0) {
        printf("adl_openData() fail: %s\n", adl_errorInfo(midi));
        ret = MR_FAILED;
    } else {
        midiPlaying = true;
    }
    SDL_UnlockAudioDevice(dev);
    return ret;
}

//////////////////////////////////////////////////////////////////////////////////////////

int32 soundInit(void) {
    SDL_AudioSpec want;
    SDL_zero(want);
    want.freq = DEV_FREQ;
    want.format = DEV_FORMAT;
    want.channels = DEV_CHANNELS;
    want.samples = DEV_SAMPLES;
    want.callback = audioCb;

    // allowed_changes 传 0，拿到的一定是上面这个格式，格式不匹配由 SDL 内部转换
    dev = SDL_OpenAudioDevice(NULL, 0, &want, &devSpec, 0);
    if (dev == 0) {
        printf("SDL_OpenAudioDevice() fail: %s\n", SDL_GetError());
        return MR_FAILED;
    }

    midiBuf = SDL_malloc(devSpec.size);
    if (midiBuf != NULL) {
        midi = adl_init(devSpec.freq);
    }
    if (midi == NULL) {
        printf("adl_init() fail: %s\n", adl_errorString());
    } else {
        // 掌机 CPU 有限，用 DosBox 这个轻量的 OPL3 实现，
        // 默认的 Nuked 精度高但太吃 CPU，一边跑 unicorn 一边合成会卡
        adl_switchEmulator(midi, ADLMIDI_EMU_DOSBOX);
        adl_setLoopEnabled(midi, 0);
    }

    if (mpg123_init() == MPG123_OK) {
        mpg123Ready = true;
    } else {
        printf("mpg123_init() fail\n");
    }

    SDL_PauseAudioDevice(dev, 0);
    return MR_SUCCESS;
}

void soundDeinit(void) {
    if (dev != 0) {
        SDL_CloseAudioDevice(dev);
        dev = 0;
    }
    for (int i = 0; i < SOUND_TYPE_COUNT; i++) {
        SDL_free(voices[i].buf);
        voices[i].buf = NULL;
    }
    if (midi != NULL) {
        adl_close(midi);
        midi = NULL;
    }
    SDL_free(midiBuf);
    midiBuf = NULL;
    if (mpg123Ready) {
        mpg123_exit();
        mpg123Ready = false;
    }
}

int32 soundPlay(int32 type, const void *data, uint32 dataLen, int32 loop) {
    if (dev == 0 || data == NULL || dataLen == 0) {
        return MR_FAILED;
    }
    switch (type) {
        case SOUND_MIDI:
            return playMidi(data, dataLen, loop);

        case SOUND_WAV: {
            SDL_RWops *rw = SDL_RWFromConstMem(data, (int)dataLen);
            if (rw == NULL) {
                return MR_FAILED;
            }
            SDL_AudioSpec spec;
            Uint8 *wav = NULL;
            Uint32 wavLen = 0;
            if (SDL_LoadWAV_RW(rw, 1, &spec, &wav, &wavLen) == NULL) {
                printf("SDL_LoadWAV_RW() fail: %s\n", SDL_GetError());
                return MR_FAILED;
            }
            Uint32 len = 0;
            Uint8 *buf = toDevFormat(wav, wavLen, spec.format, spec.channels, spec.freq, &len);
            SDL_FreeWAV(wav);
            return startVoice(SOUND_WAV, buf, len, loop);
        }

        case SOUND_MP3: {
            int freq = 0, channels = 0;
            Uint32 rawLen = 0;
            Uint8 *raw = decodeMp3(data, dataLen, &rawLen, &freq, &channels);
            if (raw == NULL) {
                return MR_FAILED;
            }
            Uint32 len = 0;
            Uint8 *buf = toDevFormat(raw, rawLen, AUDIO_S16SYS, channels, freq, &len);
            SDL_free(raw);
            return startVoice(SOUND_MP3, buf, len, loop);
        }

        case SOUND_PCM: {
            Uint32 len = 0;
            Uint8 *buf = toDevFormat(data, dataLen, MRP_PCM_FORMAT, MRP_PCM_CHANNELS, MRP_PCM_FREQ, &len);
            return startVoice(SOUND_PCM, buf, len, loop);
        }

        default:
            // AMR 没有解码器
            printf("soundPlay(): 不支持的声音类型 %d\n", type);
            return MR_FAILED;
    }
}

int32 soundStop(int32 type) {
    if (dev == 0 || type < 0 || type >= SOUND_TYPE_COUNT) {
        return MR_FAILED;
    }
    SDL_LockAudioDevice(dev);
    if (type == SOUND_MIDI) {
        midiPlaying = false;
    } else {
        SDL_free(voices[type].buf);
        voices[type].buf = NULL;
        voices[type].len = 0;
        voices[type].pos = 0;
    }
    SDL_UnlockAudioDevice(dev);
    return MR_SUCCESS;
}
