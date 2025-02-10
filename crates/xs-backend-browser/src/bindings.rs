
extern "C" {
    pub fn init_canvas();
    pub fn console_log(ptr: u32, len: u32);
    /*
    pub fn request_animation_frame(f: u32);
    pub fn setup_canvas_size(high_dpi: bool);
    pub fn run_animation_loop(blocking: bool);
    pub fn canvas_width() -> i32;
    pub fn canvas_height() -> i32;
    pub fn dpi_scale() -> f32;

    pub fn console_debug(ptr: u32, len: u32);

    pub fn console_info(ptr: u32, len: u32);
    pub fn console_warn(ptr: u32, len: u32);
    pub fn console_error(ptr: u32, len: u32);

    pub fn sapp_set_clipboard(clipboard: *const i8, len: usize);

    /// call "requestPointerLock" and "exitPointerLock" internally.
    /// Will hide cursor and will disable mouse_move events, but instead will
    /// will make inifinite mouse field for raw_device_input event.
    /// Notice that this function will works only from "engaging" event callbacks - from
    /// "mouse_down"/"key_down" event handler functions.
    pub fn sapp_set_cursor_grab(grab: bool);

    pub fn sapp_set_cursor(cursor: *const u8, len: usize);

    pub fn sapp_is_elapsed_timer_supported() -> bool;

    pub fn sapp_set_fullscreen(fullscreen: bool);
    pub fn sapp_is_fullscreen() -> bool;
    pub fn sapp_set_window_size(new_width: u32, new_height: u32);
    pub fn sapp_schedule_update();
    */
    pub fn now() -> f64;
}
