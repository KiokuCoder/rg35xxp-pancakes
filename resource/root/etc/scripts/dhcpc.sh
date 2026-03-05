#!/bin/sh

# udhcpc 会传入 $1 参数，表示当前状态（bound, renew, deconfig 等）
case "$1" in
    deconfig)
        ip addr flush dev $interface
        ;;
    bound|renew)
        # 设定 IP 和掩码
        ip addr add $ip/$mask dev $interface

        # 设定默认网关 (如果路由器传回了 router 参数)
        if [ -n "$router" ]; then
            ip route add default via $router dev $interface
        fi

        # 设定 DNS (如果路由器传回了 dns 参数)
        if [ -n "$dns" ]; then
            for i in $dns; do
                echo "nameserver $i" >> /etc/resolv.conf
            done
        fi
        ;;
esac