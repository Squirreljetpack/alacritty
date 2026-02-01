use rdev::Key;

fn code_to_rdev(code: Code) -> Key {
    match code {
        // Alphanumeric Row
        Code::Backquote => rdev::Key::BackQuote,
        Code::Digit1 => rdev::Key::Num1,
        Code::Digit2 => rdev::Key::Num2,
        Code::Digit3 => rdev::Key::Num3,
        Code::Digit4 => rdev::Key::Num4,
        Code::Digit5 => rdev::Key::Num5,
        Code::Digit6 => rdev::Key::Num6,
        Code::Digit7 => rdev::Key::Num7,
        Code::Digit8 => rdev::Key::Num8,
        Code::Digit9 => rdev::Key::Num9,
        Code::Digit0 => rdev::Key::Num0,
        Code::Minus => rdev::Key::Minus,
        Code::Equal => rdev::Key::Equal,
        Code::Backspace => rdev::Key::Backspace,

        // Alpha Keys
        Code::KeyQ => rdev::Key::KeyQ,
        Code::KeyW => rdev::Key::KeyW,
        Code::KeyE => rdev::Key::KeyE,
        Code::KeyR => rdev::Key::KeyR,
        Code::KeyT => rdev::Key::KeyT,
        Code::KeyY => rdev::Key::KeyY,
        Code::KeyU => rdev::Key::KeyU,
        Code::KeyI => rdev::Key::KeyI,
        Code::KeyO => rdev::Key::KeyO,
        Code::KeyP => rdev::Key::KeyP,
        Code::BracketLeft => rdev::Key::LeftBracket,
        Code::BracketRight => rdev::Key::RightBracket,
        Code::Backslash => rdev::Key::BackSlash,

        Code::KeyA => rdev::Key::KeyA,
        Code::KeyS => rdev::Key::KeyS,
        Code::KeyD => rdev::Key::KeyD,
        Code::KeyF => rdev::Key::KeyF,
        Code::KeyG => rdev::Key::KeyG,
        Code::KeyH => rdev::Key::KeyH,
        Code::KeyJ => rdev::Key::KeyJ,
        Code::KeyK => rdev::Key::KeyK,
        Code::KeyL => rdev::Key::KeyL,
        Code::Semicolon => rdev::Key::SemiColon,
        Code::Quote => rdev::Key::Quote,
        Code::Enter => rdev::Key::Return,

        Code::KeyZ => rdev::Key::KeyZ,
        Code::KeyX => rdev::Key::KeyX,
        Code::KeyC => rdev::Key::KeyC,
        Code::KeyV => rdev::Key::KeyV,
        Code::KeyB => rdev::Key::KeyB,
        Code::KeyN => rdev::Key::KeyN,
        Code::KeyM => rdev::Key::KeyM,
        Code::Comma => rdev::Key::Comma,
        Code::Period => rdev::Key::Dot,
        Code::Slash => rdev::Key::Slash,
        Code::IntlBackslash => rdev::Key::IntlBackslash,

        // Modifiers
        Code::ShiftLeft => rdev::Key::ShiftLeft,
        Code::ShiftRight => rdev::Key::ShiftRight,
        Code::ControlLeft => rdev::Key::ControlLeft,
        Code::ControlRight => rdev::Key::ControlRight,
        Code::AltLeft => rdev::Key::Alt,
        Code::AltRight => rdev::Key::AltGr,
        Code::MetaLeft => rdev::Key::MetaLeft,
        Code::MetaRight => rdev::Key::MetaRight,
        Code::Space => rdev::Key::Space,
        Code::Tab => rdev::Key::Tab,
        Code::CapsLock => rdev::Key::CapsLock,

        // Navigation / Editing
        Code::Insert => rdev::Key::Insert,
        Code::Delete => rdev::Key::Delete,
        Code::Home => rdev::Key::Home,
        Code::End => rdev::Key::End,
        Code::PageUp => rdev::Key::PageUp,
        Code::PageDown => rdev::Key::PageDown,
        Code::Escape => rdev::Key::Escape,

        // Arrows
        Code::ArrowUp => rdev::Key::UpArrow,
        Code::ArrowDown => rdev::Key::DownArrow,
        Code::ArrowLeft => rdev::Key::LeftArrow,
        Code::ArrowRight => rdev::Key::RightArrow,

        // Function Keys
        Code::F1 => rdev::Key::F1,
        Code::F2 => rdev::Key::F2,
        Code::F3 => rdev::Key::F3,
        Code::F4 => rdev::Key::F4,
        Code::F5 => rdev::Key::F5,
        Code::F6 => rdev::Key::F6,
        Code::F7 => rdev::Key::F7,
        Code::F8 => rdev::Key::F8,
        Code::F9 => rdev::Key::F9,
        Code::F10 => rdev::Key::F10,
        Code::F11 => rdev::Key::F11,
        Code::F12 => rdev::Key::F12,

        // Numpad
        Code::NumLock => rdev::Key::NumLock,
        Code::Numpad0 => rdev::Key::Kp0,
        Code::Numpad1 => rdev::Key::Kp1,
        Code::Numpad2 => rdev::Key::Kp2,
        Code::Numpad3 => rdev::Key::Kp3,
        Code::Numpad4 => rdev::Key::Kp4,
        Code::Numpad5 => rdev::Key::Kp5,
        Code::Numpad6 => rdev::Key::Kp6,
        Code::Numpad7 => rdev::Key::Kp7,
        Code::Numpad8 => rdev::Key::Kp8,
        Code::Numpad9 => rdev::Key::Kp9,
        Code::NumpadAdd => rdev::Key::KpPlus,
        Code::NumpadSubtract => rdev::Key::KpMinus,
        Code::NumpadMultiply => rdev::Key::KpMultiply,
        Code::NumpadDivide => rdev::Key::KpDivide,
        Code::NumpadEnter => rdev::Key::KpReturn,

        // System
        Code::PrintScreen => rdev::Key::PrintScreen,
        Code::ScrollLock => rdev::Key::ScrollLock,
        Code::Pause => rdev::Key::Pause,
        Code::Fn => rdev::Key::Function,

        // Non-standard / Others mapped to Unknown
        _ => rdev::Key::Unknown(0),
    }
}
fn key_to_code(key: Key) -> Code {
    match key {
        // Alphanumeric Row
        Key::BackQuote => Code::Backquote,
        Key::Num1 => Code::Digit1,
        Key::Num2 => Code::Digit2,
        Key::Num3 => Code::Digit3,
        Key::Num4 => Code::Digit4,
        Key::Num5 => Code::Digit5,
        Key::Num6 => Code::Digit6,
        Key::Num7 => Code::Digit7,
        Key::Num8 => Code::Digit8,
        Key::Num9 => Code::Digit9,
        Key::Num0 => Code::Digit0,
        Key::Minus => Code::Minus,
        Key::Equal => Code::Equal,
        Key::Backspace => Code::Backspace,

        // Top Row Alpha
        Key::Tab => Code::Tab,
        Key::KeyQ => Code::KeyQ,
        Key::KeyW => Code::KeyW,
        Key::KeyE => Code::KeyE,
        Key::KeyR => Code::KeyR,
        Key::KeyT => Code::KeyT,
        Key::KeyY => Code::KeyY,
        Key::KeyU => Code::KeyU,
        Key::KeyI => Code::KeyI,
        Key::KeyO => Code::KeyO,
        Key::KeyP => Code::KeyP,
        Key::LeftBracket => Code::BracketLeft,
        Key::RightBracket => Code::BracketRight,
        Key::BackSlash => Code::Backslash,

        // Home Row Alpha
        Key::CapsLock => Code::CapsLock,
        Key::KeyA => Code::KeyA,
        Key::KeyS => Code::KeyS,
        Key::KeyD => Code::KeyD,
        Key::KeyF => Code::KeyF,
        Key::KeyG => Code::KeyG,
        Key::KeyH => Code::KeyH,
        Key::KeyJ => Code::KeyJ,
        Key::KeyK => Code::KeyK,
        Key::KeyL => Code::KeyL,
        Key::SemiColon => Code::Semicolon,
        Key::Quote => Code::Quote,
        Key::Return => Code::Enter,

        // Bottom Row Alpha
        Key::ShiftLeft => Code::ShiftLeft,
        Key::KeyZ => Code::KeyZ,
        Key::KeyX => Code::KeyX,
        Key::KeyC => Code::KeyC,
        Key::KeyV => Code::KeyV,
        Key::KeyB => Code::KeyB,
        Key::KeyN => Code::KeyN,
        Key::KeyM => Code::KeyM,
        Key::Comma => Code::Comma,
        Key::Dot => Code::Period,
        Key::Slash => Code::Slash,
        Key::ShiftRight => Code::ShiftRight,

        // Bottom Bar
        Key::ControlLeft => Code::ControlLeft,
        Key::MetaLeft => Code::MetaLeft,
        Key::Alt => Code::AltLeft,
        Key::Space => Code::Space,
        Key::AltGr => Code::AltRight,
        Key::MetaRight => Code::MetaRight,
        Key::ControlRight => Code::ControlRight,

        // Navigation
        Key::Insert => Code::Insert,
        Key::Delete => Code::Delete,
        Key::Home => Code::Home,
        Key::End => Code::End,
        Key::PageUp => Code::PageUp,
        Key::PageDown => Code::PageDown,

        // Arrows
        Key::UpArrow => Code::ArrowUp,
        Key::DownArrow => Code::ArrowDown,
        Key::LeftArrow => Code::ArrowLeft,
        Key::RightArrow => Code::ArrowRight,

        // Function Keys (F1-F12)
        Key::F1 => Code::F1,
        Key::F2 => Code::F2,
        Key::F3 => Code::F3,
        Key::F4 => Code::F4,
        Key::F5 => Code::F5,
        Key::F6 => Code::F6,
        Key::F7 => Code::F7,
        Key::F8 => Code::F8,
        Key::F9 => Code::F9,
        Key::F10 => Code::F10,
        Key::F11 => Code::F11,
        Key::F12 => Code::F12,

        // Extended Function Keys (F13-F24)
        Key::F13 => Code::F13,
        Key::F14 => Code::F14,
        Key::F15 => Code::F15,
        Key::F16 => Code::F16,
        Key::F17 => Code::F17,
        Key::F18 => Code::F18,
        Key::F19 => Code::F19,
        Key::F20 => Code::F20,
        Key::F21 => Code::F21,
        Key::F22 => Code::F22,
        Key::F23 => Code::F23,
        Key::F24 => Code::F24,

        // Numpad
        Key::NumLock => Code::NumLock,
        Key::Kp0 => Code::Numpad0,
        Key::Kp1 => Code::Numpad1,
        Key::Kp2 => Code::Numpad2,
        Key::Kp3 => Code::Numpad3,
        Key::Kp4 => Code::Numpad4,
        Key::Kp5 => Code::Numpad5,
        Key::Kp6 => Code::Numpad6,
        Key::Kp7 => Code::Numpad7,
        Key::Kp8 => Code::Numpad8,
        Key::Kp9 => Code::Numpad9,
        Key::KpAdd => Code::NumpadAdd,
        Key::KpMinus => Code::NumpadSubtract,
        Key::KpMultiply => Code::NumpadMultiply,
        Key::KpDivide => Code::NumpadDivide,
        Key::KpDecimal => Code::NumpadDecimal,
        Key::KpReturn => Code::NumpadEnter,
        Key::KpEqual => Code::NumpadEqual,
        Key::KpComma => Code::NumpadComma,

        // Control Keys
        Key::Escape => Code::Escape,
        Key::PrintScreen => Code::PrintScreen,
        Key::ScrollLock => Code::ScrollLock,
        Key::Pause => Code::Pause,
        Key::Function => Code::Fn,
        Key::Apps => Code::ContextMenu,
        Key::Help => Code::Help,
        Key::Sleep => Code::Sleep,

        // International Keys
        Key::IntlBackslash => Code::IntlBackslash,
        Key::IntlRo => Code::IntlRo,
        Key::IntlYen => Code::IntlYen,
        Key::KanaMode => Code::KanaMode,
        Key::Lang1 => Code::Lang1,
        Key::Lang2 => Code::Lang2,
        Key::Lang3 => Code::Lang3,
        Key::Lang4 => Code::Lang4,
        Key::Lang5 => Code::Lang5,

        // Audio
        Key::VolumeUp => Code::AudioVolumeUp,
        Key::VolumeDown => Code::AudioVolumeDown,
        Key::VolumeMute => Code::AudioVolumeMute,

        // Miscellaneous Mappings
        Key::Cancel => Code::Abort,
        Key::Select => Code::Select,
        Key::Execute => Code::Resume, // Best effort logical mapping
        Key::Print => Code::Props,    // Print often maps to Props on Sun/Linux
        Key::Clear => Code::NumpadClear,

        // Catch-all for variants without a direct Code equivalent
        _ => Code::Unidentified,
    }
}
