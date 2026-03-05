#!/usr/bin/bash

[ -e /proc/sys/kernel/hotplug ] && printf '\000\000\000\000' > /proc/sys/kernel/hotplug
/usr/sbin/udevd -d || { echo "FAIL"; exit 1; }
udevadm trigger --type=subsystems --action=add
udevadm trigger --type=devices --action=add
