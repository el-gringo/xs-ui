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

static SAPP_MODIFIER_SHIFT: u8 = 1;
static SAPP_MODIFIER_CTRL: u8 = 2;
static SAPP_MODIFIER_ALT: u8 = 4;
static SAPP_MODIFIER_SUPER: u8 = 8;

pub fn _translate_mod(wasm_mods: u8) -> KeyMods {
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

/// `KeyboardEvent.code` to [`KeyCode`]. Keyed on the physical key, so it does
/// not depend on the layout: `ArrowUp` is the same key everywhere.
pub fn _translate_code(code: &str) -> KeyCode {
    match code {
        "Enter" | "NumpadEnter" => KeyCode::Enter,
        "Backspace" => KeyCode::Backspace,
        "ArrowRight" => KeyCode::Right,
        "ArrowLeft" => KeyCode::Left,
        "ArrowDown" => KeyCode::Down,
        "ArrowUp" => KeyCode::Up,
        "Space" => KeyCode::Space,
        "Escape" => KeyCode::Escape,
        "PageUp" => KeyCode::PageUp,
        "PageDown" => KeyCode::PageDown,
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
