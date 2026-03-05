use log::warn;

pub type EGLint = i32;
pub type EGLBoolean = std::ffi::c_uint;
pub type EGLDisplay = *mut std::ffi::c_void;
pub type EGLConfig = *mut std::ffi::c_void;

pub type EGLSurface = *mut std::ffi::c_void;
pub type EGLContext = *mut std::ffi::c_void;
pub type EGLNativePixmapType = i32;
pub type EGLNativeWindowType = *mut std::ffi::c_void;
pub type EGLNativeDisplayType = *mut std::ffi::c_void;
pub type NativeDisplayType = EGLNativeDisplayType;
pub type NativePixmapType = EGLNativePixmapType;
pub type NativeWindowType = EGLNativeWindowType;
pub type EGLenum = std::ffi::c_uint;
pub type EGLClientBuffer = *mut std::ffi::c_void;
pub type EGLSync = *mut std::ffi::c_void;
pub type EGLAttrib = isize;
pub type EGLTime = u64;
pub type EGLImage = *mut std::ffi::c_void;
pub const EGL_NO_CONTEXT: EGLContext = 0 as *mut std::ffi::c_void;
pub const EGL_NO_DISPLAY: EGLDisplay = 0 as *mut std::ffi::c_void;
pub const EGL_NO_SURFACE: EGLSurface = 0 as *mut std::ffi::c_void;
pub const EGL_TRUE: EGLBoolean = 1;
pub const EGL_FALSE: EGLBoolean = 0;
pub const EGL_ALPHA_SIZE: EGLint = 0x3021;
pub const EGL_BLUE_SIZE: EGLint = 0x3022;
pub const EGL_GREEN_SIZE: EGLint = 0x3023;
pub const EGL_RED_SIZE: EGLint = 0x3024;
pub const EGL_DEPTH_SIZE: EGLint = 0x3025;
pub const EGL_STENCIL_SIZE: EGLint = 0x3026;
pub const EGL_NONE: EGLint = 0x3038;
pub const EGL_OPENGL_ES_API: EGLenum = 0x30A0;
pub const EGL_HEIGHT: EGLint = 0x3056;
pub const EGL_WIDTH: EGLint = 0x3057;
pub const EGL_CONTEXT_CLIENT_VERSION: EGLint = 0x3098;

#[link(name = "EGL")]
#[link(name = "mali")]
#[link(name = "GLESv2")]
unsafe extern "C" {
    fn eglGetDisplay(display_id: EGLNativeDisplayType) -> EGLDisplay;
    fn eglInitialize(dpy: EGLDisplay, major: *mut EGLint, minor: *mut EGLint) -> EGLBoolean;

    fn eglChooseConfig(
        dpy: EGLDisplay,
        attrib_list: *const EGLint,
        configs: *mut EGLConfig,
        config_size: EGLint,
        num_config: *mut EGLint,
    ) -> EGLBoolean;

    pub(crate) fn eglCreateWindowSurface(
        dpy: EGLDisplay,
        config: EGLConfig,
        win: EGLNativeWindowType,
        attrib_list: *const EGLint,
    ) -> EGLSurface;

    fn eglCreateContext(
        dpy: EGLDisplay,
        config: EGLConfig,
        share_context: EGLContext,
        attrib_list: *const EGLint,
    ) -> EGLContext;
    fn eglBindAPI(api: EGLenum) -> EGLBoolean;

    fn eglMakeCurrent(
        dpy: EGLDisplay,
        draw: EGLSurface,
        read: EGLSurface,
        ctx: EGLContext,
    ) -> EGLBoolean;

    fn eglDestroyContext(dpy: EGLDisplay, ctx: EGLContext) -> EGLBoolean;
    fn eglDestroySurface(dpy: EGLDisplay, surface: EGLSurface) -> EGLBoolean;
    fn eglTerminate(dpy: EGLDisplay) -> EGLBoolean;
    fn eglSwapBuffers(dpy: EGLDisplay, surface: EGLSurface) -> EGLBoolean;
    pub(crate) fn eglGetProcAddress(proc_name: *const std::ffi::c_char) -> *const std::ffi::c_void;
    fn eglQuerySurface(
        dpy: EGLDisplay,
        surface: EGLSurface,
        attribute: EGLint,
        value: *mut EGLint,
    ) -> EGLBoolean;

    fn eglGetError() -> EGLint;
    fn eglSwapInterval(dpy: EGLDisplay, interval: EGLint) -> EGLBoolean;
}
pub struct FramebufferWindow {
    major: EGLint,
    minor: EGLint,
    width: EGLint,
    height: EGLint,
    pub(crate) display: EGLDisplay,
    pub(crate) surface: EGLSurface,
    pub(crate) context: EGLContext,
}
impl FramebufferWindow {
    pub unsafe fn new() -> Result<Self, &'static str> { unsafe {
        let display = eglGetDisplay(0 as *mut std::ffi::c_void);
        if display.is_null() {
            return Err("Failed to get display");
        }
        let mut ret = FramebufferWindow {
            major: 0,
            minor: 0,
            width: 0,
            height: 0,
            display,
            surface: Default::default(),
            context: Default::default(),
        };
        if eglInitialize(display, &mut ret.major, &mut ret.minor) != EGL_TRUE {
            return Err("Failed to initialize EGL");
        }

        let mut config = std::ptr::null_mut();
        let mut config_count = 0;
        let attrib_list = [
            EGL_RED_SIZE, 8,
            EGL_GREEN_SIZE, 8,
            EGL_BLUE_SIZE, 8,
            EGL_ALPHA_SIZE, 8,
            EGL_DEPTH_SIZE, 24,
            EGL_STENCIL_SIZE, 8,
            EGL_NONE,
        ];
        if eglChooseConfig(
            display,
            attrib_list.as_ptr(),
            &mut config,
            1,
            &mut config_count,
        ) != EGL_TRUE
        {
            return Err("Failed to choose config");
        }
        if config.is_null() {
            return Err("Failed to get config");
        }

        if eglBindAPI(EGL_OPENGL_ES_API) != EGL_TRUE {
            return Err("Failed to bind OpenGL API");
        }

        ret.surface =
            eglCreateWindowSurface(display, config, 0 as EGLNativeWindowType, std::ptr::null());
        if ret.surface.is_null() {
            return Err("Failed to create window surface");
        }

        if eglQuerySurface(display, ret.surface, EGL_WIDTH, &mut ret.width as *mut _) != EGL_TRUE {
            panic!("Failed to query surface width");
        }
        if eglQuerySurface(display, ret.surface, EGL_HEIGHT, &mut ret.height as *mut _) != EGL_TRUE
        {
            panic!("Failed to query surface height");
        }

        let attr = [EGL_CONTEXT_CLIENT_VERSION, 3, EGL_NONE];
        ret.context = eglCreateContext(display, config, EGL_NO_CONTEXT, attr.as_ptr());
        if ret.context.is_null() {
            return Err("Failed to create context");
        }
        ret.make_current()?;
        Ok(ret)
    }}
    pub fn make_current(&self) -> Result<(), &'static str> {
        if unsafe { eglMakeCurrent(self.display, self.surface, self.surface, self.context) }
            != EGL_TRUE
        {
            return Err("Failed to make context current");
        }
        Ok(())
    }
    pub fn set_swap_interval(&self, interval: i32) -> Result<(), &'static str> {
        // 注意：调用 eglSwapInterval 之前必须保证 Context 已经是 Current 状态
        // 你的代码在 new() 的最后已经调用了 make_current()，所以可以直接调用
        unsafe {
            if eglSwapInterval(self.display, interval as EGLint) == EGL_TRUE {
                Ok(())
            } else {
                Err("Failed to set swap interval")
            }
        }
    }

    pub fn present(&self) {
        unsafe {
            eglSwapBuffers(self.display, self.surface);
        }
    }

    pub fn size(&self) -> (u32, u32) {
        (self.width as u32, self.height as u32)
    }
    pub fn check_error(&self) {
        unsafe {
            let err = eglGetError();
            if err != 0x3000 { // EGL_SUCCESS = 0x3000
                warn!("EGL Error: {:x}", err);
            }
        }
    }
}

pub fn get_proc_address(fn_name: &str) -> *const std::ffi::c_void {
    unsafe {
        let cstr = std::ffi::CString::new(fn_name).unwrap();
        eglGetProcAddress(cstr.as_ptr()) as *const _
    }
}

impl Drop for FramebufferWindow {
    fn drop(&mut self) {
        unsafe {
            if !self.display.is_null() {
                eglMakeCurrent(self.display, EGL_NO_SURFACE, EGL_NO_SURFACE, EGL_NO_CONTEXT);
                if !self.context.is_null() {
                    eglDestroyContext(self.display, self.context);
                }
                if !self.surface.is_null() {
                    eglDestroySurface(self.display, self.surface);
                }
                eglTerminate(self.display);
            }
        }
    }
}
