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
pub const EGL_BLUE_SIZE: EGLint = 0x3022;
pub const EGL_GREEN_SIZE: EGLint = 0x3023;
pub const EGL_RED_SIZE: EGLint = 0x3024;
pub const EGL_NONE: EGLint = 0x3038;
pub const EGL_OPENGL_ES_API: EGLenum = 0x30A0;
pub const EGL_HEIGHT: EGLint = 0x3056;
pub const EGL_WIDTH: EGLint = 0x3057;
pub const EGL_CONTEXT_CLIENT_VERSION: EGLint = 0x3098;

#[link(name = "EGL")]
#[link(name = "mali")]
#[link(name = "GLESv2")]
extern "C" {
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
}
pub struct FramebufferWindow {
    width: u32,
    height: u32,
    display: EGLDisplay,
    surface: EGLSurface,
    context: EGLContext,
}
impl FramebufferWindow {
    pub unsafe fn new() -> Self {
        let display = eglGetDisplay(0 as *mut std::ffi::c_void);
        if display.is_null() {
            panic!("Failed to get display");
        }
        let mut major = 0;
        let mut minor = 0;
        if eglInitialize(display, &mut major, &mut minor) != EGL_TRUE {
            panic!("Failed to initialize EGL");
        }

        let mut config = std::ptr::null_mut();
        let mut config_count = 0;
        let attrib_list = [
            EGL_RED_SIZE,
            8,
            EGL_GREEN_SIZE,
            8,
            EGL_BLUE_SIZE,
            8,
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
            panic!("Failed to choose config");
        }
        if config.is_null() {
            panic!("Failed to get config");
        }

        if eglBindAPI(EGL_OPENGL_ES_API) != EGL_TRUE {
            panic!("Failed to bind OpenGL API");
        }

        let surface =
            eglCreateWindowSurface(display, config, 0 as EGLNativeWindowType, std::ptr::null());
        if surface.is_null() {
            panic!("Failed to create window surface");
        }
        let mut width: EGLint = 0;
        let mut height: EGLint = 0;
        if eglQuerySurface(display, surface, EGL_WIDTH, &mut width as *mut _) != EGL_TRUE {
            panic!("Failed to query surface width");
        }
        if eglQuerySurface(display, surface, EGL_HEIGHT, &mut height as *mut _) != EGL_TRUE {
            panic!("Failed to query surface height");
        }


        let attr = [EGL_CONTEXT_CLIENT_VERSION, 2, EGL_NONE];
        let context = eglCreateContext(display, config, EGL_NO_CONTEXT, attr.as_ptr());
        if context.is_null() {
            panic!("Failed to create context");
        }
        if eglMakeCurrent(display, surface, surface, context) != EGL_TRUE {
            panic!("Failed to make context current");
        }
        FramebufferWindow {
            width: width as u32,
            height: height as u32,
            display,
            surface,
            context,
        }
    }

    pub fn present(&self) {
        unsafe {
            eglSwapBuffers(self.display, self.surface);
        }
    }

    pub fn get_proc_address(&self, fn_name: &str) -> *const std::ffi::c_void {
        unsafe {
            let cstr = std::ffi::CString::new(fn_name).unwrap();
            eglGetProcAddress(cstr.as_ptr()) as *const _
        }
    }

    pub fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }
}

impl Drop for FramebufferWindow {
    fn drop(&mut self) {
        unsafe {
            eglDestroyContext(self.display, self.context);
            eglDestroySurface(self.display, self.surface);
            eglTerminate(self.display);
        }
    }
}
