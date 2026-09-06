# vmrp 交叉编译（aarch64），由 package/vmrp.ts 调用。
# 源码整理自 https://github.com/vmrp/vmrp ，改动见 README.md。

CROSS_COMPILE ?= aarch64-linux-gnu-
CC := $(CROSS_COMPILE)gcc
PKG_CONFIG ?= $(CROSS_COMPILE)pkg-config

# libADLMIDI 和 libmpg123 是 sound.c 用的 MIDI/MP3 解码器
PKGS := sdl2 zlib libADLMIDI libmpg123

# vmrp 找不到 cfunction.ext 时会 chdir 到这里，见 main.c
VMRP_HOME ?= /root/apps/vmrp
BUILD_DIR ?= build

SRCS := main.c bridge.c fileLib.c memory.c network.c rbtree.c sound.c utils.c vmrp.c
OBJS := $(SRCS:%.c=$(BUILD_DIR)/%.o)

# -w: 上游代码的警告非常多，交叉编译时全部关掉
CFLAGS := -O2 -w -DVMRP -DNETWORK_SUPPORT -DVMRP_HOME='"$(VMRP_HOME)"' \
	$(shell $(PKG_CONFIG) --cflags $(PKGS))
LDLIBS := $(shell $(PKG_CONFIG) --libs $(PKGS)) -lunicorn -lpthread -lm

all: $(BUILD_DIR)/vmrp

$(BUILD_DIR)/vmrp: $(OBJS)
	$(CC) -o $@ $^ $(LDFLAGS) $(LDLIBS)

$(BUILD_DIR)/%.o: %.c
	@mkdir -p $(dir $@)
	$(CC) $(CFLAGS) -c -o $@ $<

clean:
	rm -rf $(BUILD_DIR)

.PHONY: all clean
