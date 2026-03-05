# rg35xx-plus egui

编译需要用到对应的 SDK，
还有 GPU 驱动 : [t507_gpu_drivers](https://github.com/knulli-cfw/t507_gpu_drivers)

### 说明
测试移植 egui 到 rg35xxp 上运行，测试没有问题，但是因为 egui 特有的焦点系统与设备按键操作不匹配，所以改用其他 UI 库，如果有兴趣可以研究看看。