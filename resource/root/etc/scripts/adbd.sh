#!/bin/sh

echo "Starting adb configuration..."

# 1. 准备 ConfigFS 基础结构
mount -t configfs none /sys/kernel/config 2>/dev/null
mkdir -p /sys/kernel/config/usb_gadget/g1
cd /sys/kernel/config/usb_gadget/g1

# 2. 设置 USB 参数 (2.0)
echo 0x0200 > bcdUSB
echo 1 > os_desc/use

# 3. 设置设备身份信息
mkdir -p strings/0x409
echo "0x1d6b" > idVendor
echo "0x0104" > idProduct
echo "0123456789" > strings/0x409/serialnumber
echo "Foo Inc" > strings/0x409/manufacturer
echo "Bar Prod" > strings/0x409/product

# 4. 创建 ADB 功能实例与配置绑定
mkdir -p functions/ffs.adb
mkdir -p configs/b.1/strings/0x409
echo "ffs.adb" > configs/b.1/strings/0x409/configuration
ln -s functions/ffs.adb configs/b.1/

# 5. 挂载 FunctionFS
# adbd 依赖于这个文件系统来与内核通信
mkdir -p /dev/usb-ffs/adb
mount -t functionfs adb /dev/usb-ffs/adb

# 6. 配置 Windows 兼容性描述符
echo 0x1 > os_desc/b_vendor_code

# 7. 激活 USB 控制器 (UDC)
# 必须先执行这一步，将逻辑设备绑定到物理控制器
# 注意：请确保 5100000.udc-controller 路径正确
UDC_DEV="5100000.udc-controller"
if [ -e "/sys/class/udc/$UDC_DEV" ]; then
    echo "$UDC_DEV" > UDC
    echo "UDC device bound: OK"
else
    echo "Error: UDC device $UDC_DEV not found!"
    # 尝试自动查找第一个可用的 UDC
    ls /sys/class/udc/ > UDC
fi

echo "All configurations set. Starting adbd daemon in foreground..."

# 8. 前台启动 adbd
# 去掉了 start-stop-daemon，直接调用二进制文件
# 使用 exec 可以让 adbd 进程替换当前的 shell 进程
exec /usr/bin/adbd