#!/bin/sh

# wpa_cli 会传递两个参数：
# $1: 网卡接口名称 (如 wlan0)
# $2: 事件类型 (如 CONNECTED, DISCONNECTED, TERMINATING)

INTERFACE=$1
ACTION=$2

case "$ACTION" in
    CONNECTED)
        echo "Network connected on $INTERFACE, starting udhcpc..."
        # -i: 指定接口
        # -n: 如果没获取到 IP 则退出（可选）
        # -q: 获取到 IP 后退出（如果你希望它作为守护进程运行，去掉 -q）
        # -R: 释放租约后退出
        busybox udhcpc -i "$INTERFACE" -s /etc/scripts/dhcpc.sh -n -q
        ;;
    DISCONNECTED)
        echo "Network disconnected on $INTERFACE."
        # 可选：断开连接时释放 IP 或停止相关进程
        # killall udhcpc
        ip addr flush dev "$INTERFACE"
        ;;
    *)
        echo "Ignore action: $ACTION"
        ;;
esac