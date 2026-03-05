use std::fs::File;
use std::os::unix::io::AsRawFd;
use image::{GenericImageView, Pixel};
use nix::sys::mman::{mmap, MapFlags, ProtFlags};
use std::num::NonZeroUsize;

// 手动定义 FB 相关的结构体和常量，因为有些平台 libc 没有
#[repr(C)]
#[derive(Default)]
struct FbBitfield {
    offset: u32,
    length: u32,
    msb_right: u32,
}

#[repr(C)]
#[derive(Default)]
struct FbVarScreeninfo {
    xres: u32,
    yres: u32,
    xres_virtual: u32,
    yres_virtual: u32,
    xoffset: u32,
    yoffset: u32,
    bits_per_pixel: u32,
    grayscale: u32,
    red: FbBitfield,
    green: FbBitfield,
    blue: FbBitfield,
    transp: FbBitfield,
    nonstd: u32,
    activate: u32,
    height: u32,
    width: u32,
    accel_flags: u32,
    pixclock: u32,
    left_margin: u32,
    right_margin: u32,
    upper_margin: u32,
    lower_margin: u32,
    hsync_len: u32,
    vsync_len: u32,
    sync: u32,
    vmode: u32,
    rotate: u32,
    colorspace: u32,
    reserved: [u32; 4],
}

const FBIOGET_VSCREENINFO: libc::c_ulong = 0x4600;

pub fn display_logo() {
    // 嵌入图片
    let img_data = include_bytes!("logo.png");
    
    // 打开 framebuffer 设备
    let fb_file = match File::options().read(true).write(true).open("/dev/fb0") {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Failed to open /dev/fb0: {}", e);
            return;
        }
    };

    let fd = fb_file.as_raw_fd();

    let mut vinfo = FbVarScreeninfo::default();
    if unsafe { libc::ioctl(fd, FBIOGET_VSCREENINFO, &mut vinfo) } < 0 {
        eprintln!("Failed to get fb_var_screeninfo");
        return;
    }

    let width = vinfo.xres;
    let height = vinfo.yres;
    let bpp = vinfo.bits_per_pixel;
    let line_length = width * (bpp / 8);
    let size = (line_length * height) as usize;

    if size == 0 { return; }

    // 映射内存
    let mmap_ptr = unsafe {
        mmap(
            None,
            NonZeroUsize::new(size).unwrap(),
            ProtFlags::PROT_READ | ProtFlags::PROT_WRITE,
            MapFlags::MAP_SHARED,
            &fb_file,
            0,
        )
    };

    let mmap_ptr = match mmap_ptr {
        Ok(p) => p.as_ptr() as *mut u8,
        Err(e) => {
            eprintln!("Failed to mmap fb: {}", e);
            return;
        }
    };

    // 解码图片
    let img = match image::load_from_memory(img_data) {
        Ok(i) => i,
        Err(e) => {
            eprintln!("Failed to decode logo.png: {}", e);
            unsafe { libc::munmap(mmap_ptr as *mut libc::c_void, size); }
            return;
        }
    };

    let (img_w, img_h) = img.dimensions();
    
    // 居中显示计算
    let start_x = if width > img_w { (width - img_w) / 2 } else { 0 };
    let start_y = if height > img_h { (height - img_h) / 2 } else { 0 };

    let draw_w = img_w.min(width);
    let draw_h = img_h.min(height);

    // 写入像素
    for y in 0..draw_h {
        for x in 0..draw_w {
            let pixel = img.get_pixel(x, y);
            let rgba = pixel.to_rgba();
            
            let fb_idx = ((start_y + y) * width + (start_x + x)) as usize * (bpp as usize / 8);
            
            if fb_idx + 3 < size {
                unsafe {
                    if bpp == 32 {
                        // 常见 Linux FB 是 BGRA
                        *mmap_ptr.add(fb_idx) = rgba[2];     // B
                        *mmap_ptr.add(fb_idx + 1) = rgba[1]; // G
                        *mmap_ptr.add(fb_idx + 2) = rgba[0]; // R
                        *mmap_ptr.add(fb_idx + 3) = 255;     // A
                    } else if bpp == 16 {
                        // RGB565
                        let r = (rgba[0] >> 3) as u16;
                        let g = (rgba[1] >> 2) as u16;
                        let b = (rgba[2] >> 3) as u16;
                        let rgb565 = (r << 11) | (g << 5) | b;
                        let ptr_16 = mmap_ptr.add(fb_idx) as *mut u16;
                        *ptr_16 = rgb565;
                    }
                }
            }
        }
    }

    // 刷新并清理
    unsafe {
        libc::msync(mmap_ptr as *mut libc::c_void, size, libc::MS_SYNC);
        libc::munmap(mmap_ptr as *mut libc::c_void, size);
    }
}
