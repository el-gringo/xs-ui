use xs_platform_event::{KeyCode, KeyMods, MouseButton, TouchPhase};

#[allow(dead_code)]
pub fn translate_mouse_button(button: i32) -> MouseButton {
    match button {
        0 => MouseButton::Left,
        1 => MouseButton::Right,
        2 => MouseButton::Middle,
        _ => MouseButton::Unknown,
    }
}
pub fn translate_mod(wasm_mods: i32) -> KeyMods {
    const SAPP_MODIFIER_SHIFT: i32 = 1;
    const SAPP_MODIFIER_CTRL: i32 = 2;
    const SAPP_MODIFIER_ALT: i32 = 4;
    const SAPP_MODIFIER_SUPER: i32 = 8;

    let mut mods = KeyMods::default();
    if wasm_mods & SAPP_MODIFIER_SHIFT != 0 {
        mods.shift = true;
    }
    if wasm_mods & SAPP_MODIFIER_CTRL != 0 {
        mods.ctrl = true;
    }
    if wasm_mods & SAPP_MODIFIER_ALT != 0 {
        mods.alt = true;
    }
    if wasm_mods & SAPP_MODIFIER_SUPER != 0 {
        mods.logo = true;
    }
    mods
}

pub fn translate_keycode(keycode: i32) -> KeyCode {
    match keycode {
        8 => KeyCode::Backspace,
        13 => KeyCode::Enter,
        39 => KeyCode::Right,
        37 => KeyCode::Left,
        40 => KeyCode::Down,
        38 => KeyCode::Up,
        _ => KeyCode::Unknown,
    }
}

#[allow(dead_code)]
pub fn translate_touch_phase(phase: i32) -> TouchPhase {
    const TOUCHES_BEGAN: i32 = 10;
    const TOUCHES_MOVED: i32 = 11;
    const TOUCHES_ENDED: i32 = 12;
    const TOUCHES_CANCELLED: i32 = 13;

    match phase {
        TOUCHES_BEGAN => TouchPhase::Started,
        TOUCHES_MOVED => TouchPhase::Moved,
        TOUCHES_ENDED => TouchPhase::Ended,
        TOUCHES_CANCELLED => TouchPhase::Cancelled,
        _ => TouchPhase::Moved,
    }
}
