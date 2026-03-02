use std::fmt::{self, Debug, Display};
use std::rc::Rc;

use super::bindings::{BindingKey, BindingMode, KeyBinding, KeyLocation, MouseBinding, MouseEvent};
use alacritty_terminal::vi_mode::ViMotion;
use winit::event::MouseButton;
use winit::keyboard::{Key, ModifiersState, NamedKey};

use super::hint::Hint;
use super::types::Program;

#[derive(serde::Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// Write an escape sequence.
    #[serde(skip)]
    Esc(String),

    /// Run given command.
    #[serde(skip)]
    Command(Program),

    Window(WindowAction),

    /// Regex keyboard hints.
    #[serde(skip)]
    Hint(Rc<Hint>),

    /// Move vi mode cursor.
    #[serde(skip)]
    ViMotion(ViMotion),

    /// Perform vi mode action.
    #[serde(skip)]
    Vi(ViAction),

    /// Perform search mode action.
    #[serde(skip)]
    Search(SearchAction),

    /// Perform mouse binding exclusive action.
    #[serde(skip)]
    Mouse(MouseAction),

    /// Paste contents of system clipboard.
    Paste,

    /// Store current selection into clipboard.
    Copy,

    /// Store current selection into selection buffer.
    CopySelection,

    /// Paste contents of selection buffer.
    PasteSelection,

    /// Increase font size.
    IncreaseFontSize,

    /// Decrease font size.
    DecreaseFontSize,

    /// Reset font size to the config value.
    ResetFontSize,

    /// Scroll exactly one page up.
    ScrollPageUp,

    /// Scroll exactly one page down.
    ScrollPageDown,

    /// Scroll half a page up.
    ScrollHalfPageUp,

    /// Scroll half a page down.
    ScrollHalfPageDown,

    /// Scroll one line up.
    ScrollLineUp,

    /// Scroll one line down.
    ScrollLineDown,

    /// Scroll all the way to the top.
    ScrollToTop,

    /// Scroll all the way to the bottom.
    ScrollToBottom,

    /// Clear the display buffer(s) to remove history.
    ClearHistory,

    /// Clear active selection.
    ClearSelection,

    /// Toggle vi mode.
    ToggleViMode,

    /// Allow receiving char input.
    ReceiveChar,

    /// Start a forward buffer search.
    SearchForward,

    /// Start a backward buffer search.
    SearchBackward,

    /// Clear warning and error notices.
    ClearLogNotice,

    /// No action.
    None,
}

/// Window actions.
#[allow(clippy::enum_variant_names)]
#[derive(serde::Deserialize, serde::Serialize, Debug, Copy, Clone, PartialEq, Eq)]
pub enum WindowAction {
    // global
    /// Focus the current window.
    Focus,

    /// Toggle the current window.
    Toggle,

    // local
    /// Hide the current window.
    Hide,

    /// Cycle to the next window.
    // Cycle,

    /// Close the current window
    Quit,

    /// Toggle maximized.
    ToggleMaximized,

    /// Open settings gui.
    Settings,
}

impl From<&'static str> for Action {
    fn from(s: &'static str) -> Action {
        Action::Esc(s.into())
    }
}

impl From<ViAction> for Action {
    fn from(action: ViAction) -> Self {
        Self::Vi(action)
    }
}

impl From<ViMotion> for Action {
    fn from(motion: ViMotion) -> Self {
        Self::ViMotion(motion)
    }
}

impl From<SearchAction> for Action {
    fn from(action: SearchAction) -> Self {
        Self::Search(action)
    }
}

impl From<MouseAction> for Action {
    fn from(action: MouseAction) -> Self {
        Self::Mouse(action)
    }
}

/// Display trait used for error logging.
impl Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Action::ViMotion(motion) => motion.fmt(f),
            Action::Vi(action) => action.fmt(f),
            Action::Mouse(action) => action.fmt(f),
            _ => write!(f, "{self:?}"),
        }
    }
}

/// Vi mode specific actions.
#[derive(serde::Deserialize, Debug, Copy, Clone, PartialEq, Eq)]
pub enum ViAction {
    /// Toggle normal vi selection.
    ToggleNormalSelection,
    /// Toggle line vi selection.
    ToggleLineSelection,
    /// Toggle block vi selection.
    ToggleBlockSelection,
    /// Toggle semantic vi selection.
    ToggleSemanticSelection,
    /// Jump to the beginning of the next match.
    SearchNext,
    /// Jump to the beginning of the previous match.
    SearchPrevious,
    /// Jump to the next start of a match to the left of the origin.
    SearchStart,
    /// Jump to the next end of a match to the right of the origin.
    SearchEnd,
    /// Launch the URL below the vi mode cursor.
    Open,
    /// Centers the screen around the vi mode cursor.
    CenterAroundViCursor,
    /// Search forward within the current line.
    InlineSearchForward,
    /// Search backward within the current line.
    InlineSearchBackward,
    /// Search forward within the current line, stopping just short of the character.
    InlineSearchForwardShort,
    /// Search backward within the current line, stopping just short of the character.
    InlineSearchBackwardShort,
    /// Jump to the next inline search match.
    InlineSearchNext,
    /// Jump to the previous inline search match.
    InlineSearchPrevious,
    /// Search forward for selection or word under the cursor.
    SemanticSearchForward,
    /// Search backward for selection or word under the cursor.
    SemanticSearchBackward,
}

/// Search mode specific actions.
#[allow(clippy::enum_variant_names)]
#[derive(serde::Deserialize, Debug, Copy, Clone, PartialEq, Eq)]
pub enum SearchAction {
    /// Move the focus to the next search match.
    SearchFocusNext,
    /// Move the focus to the previous search match.
    SearchFocusPrevious,
    /// Confirm the active search.
    SearchConfirm,
    /// Cancel the active search.
    SearchCancel,
    /// Reset the search regex.
    SearchClear,
    /// Delete the last word in the search regex.
    SearchDeleteWord,
    /// Go to the previous regex in the search history.
    SearchHistoryPrevious,
    /// Go to the next regex in the search history.
    SearchHistoryNext,
}

/// Mouse binding specific actions.
#[derive(serde::Deserialize, Debug, Copy, Clone, PartialEq, Eq)]
pub enum MouseAction {
    /// Expand the selection to the current mouse cursor position.
    ExpandSelection,
}

macro_rules! bindings {
    (
        $ty:ident;
        $(
            $key:tt$(::$button:ident)?
            $(=>$location:expr)?
            $(,$mods:expr)*
            $(,+$mode:expr)*
            $(,~$notmode:expr)*
            ;$action:expr
        );*
        $(;)*
    ) => {{
        let mut v = Vec::new();

        $(
            let mut _mods = ModifiersState::empty();
            $(_mods = $mods;)*
            let mut _mode = BindingMode::empty();
            $(_mode.insert($mode);)*
            let mut _notmode = BindingMode::empty();
            $(_notmode.insert($notmode);)*

            v.push($ty {
                trigger: trigger!($ty, $key$(::$button)?, $($location)?),
                mods: _mods,
                mode: _mode,
                notmode: _notmode,
                action: $action.into(),
            });
        )*

        v
    }};
}

macro_rules! trigger {
    (KeyBinding, $key:literal, $location:expr) => {{ BindingKey::Keycode { key: Key::Character($key.into()), location: $location } }};
    (KeyBinding, $key:literal,) => {{ BindingKey::Keycode { key: Key::Character($key.into()), location: KeyLocation::Any } }};
    (KeyBinding, $key:ident, $location:expr) => {{ BindingKey::Keycode { key: Key::Named(NamedKey::$key), location: $location } }};
    (KeyBinding, $key:ident,) => {{ BindingKey::Keycode { key: Key::Named(NamedKey::$key), location: KeyLocation::Any } }};
    (MouseBinding, MouseButton::$button:ident,) => {{ MouseEvent::Button(MouseButton::$button) }};
    (MouseBinding, MouseEvent::$event:ident,) => {{ MouseEvent::$event }};
}

pub fn default_mouse_bindings() -> Vec<MouseBinding> {
    bindings!(
        MouseBinding;
        MouseButton::Right;                            MouseAction::ExpandSelection;
        MouseButton::Right,   ModifiersState::CONTROL; MouseAction::ExpandSelection;
        MouseButton::Middle, ~BindingMode::VI;         Action::PasteSelection;
    )
}

// NOTE: key sequences which are not present here, like F5-F20, PageUp/PageDown codes are
// built on the fly in input/keyboard.rs.
pub fn default_key_bindings() -> Vec<KeyBinding> {
    let mut bindings = bindings!(
        KeyBinding;
        Copy; Action::Copy;
        Copy,  +BindingMode::VI; Action::ClearSelection;
        Paste, ~BindingMode::VI; Action::Paste;
        Paste, +BindingMode::VI, +BindingMode::SEARCH; Action::Paste;
        "l",       ModifiersState::CONTROL; Action::ClearLogNotice;
        "l",       ModifiersState::CONTROL; Action::ReceiveChar;
        Home,      ModifiersState::SHIFT, ~BindingMode::ALT_SCREEN; Action::ScrollToTop;
        End,       ModifiersState::SHIFT, ~BindingMode::ALT_SCREEN; Action::ScrollToBottom;
        PageUp,    ModifiersState::SHIFT, ~BindingMode::ALT_SCREEN; Action::ScrollPageUp;
        PageDown,  ModifiersState::SHIFT, ~BindingMode::ALT_SCREEN; Action::ScrollPageDown;
        // App cursor mode.
        Home,       +BindingMode::APP_CURSOR, ~BindingMode::VI, ~BindingMode::SEARCH; Action::Esc("\x1bOH".into());
        End,        +BindingMode::APP_CURSOR, ~BindingMode::VI, ~BindingMode::SEARCH; Action::Esc("\x1bOF".into());
        ArrowUp,    +BindingMode::APP_CURSOR, ~BindingMode::VI, ~BindingMode::SEARCH; Action::Esc("\x1bOA".into());
        ArrowDown,  +BindingMode::APP_CURSOR, ~BindingMode::VI, ~BindingMode::SEARCH; Action::Esc("\x1bOB".into());
        ArrowRight, +BindingMode::APP_CURSOR, ~BindingMode::VI, ~BindingMode::SEARCH; Action::Esc("\x1bOC".into());
        ArrowLeft,  +BindingMode::APP_CURSOR, ~BindingMode::VI, ~BindingMode::SEARCH; Action::Esc("\x1bOD".into());
        // Legacy keys handling which can't be automatically encoded.
        F1,         ~BindingMode::VI, ~BindingMode::SEARCH, ~BindingMode::REPORT_ALL_KEYS_AS_ESC, ~BindingMode::DISAMBIGUATE_ESC_CODES; Action::Esc("\x1bOP".into());
        F2,         ~BindingMode::VI, ~BindingMode::SEARCH, ~BindingMode::REPORT_ALL_KEYS_AS_ESC, ~BindingMode::DISAMBIGUATE_ESC_CODES; Action::Esc("\x1bOQ".into());
        F3,         ~BindingMode::VI, ~BindingMode::SEARCH, ~BindingMode::REPORT_ALL_KEYS_AS_ESC, ~BindingMode::DISAMBIGUATE_ESC_CODES; Action::Esc("\x1bOR".into());
        F4,         ~BindingMode::VI, ~BindingMode::SEARCH, ~BindingMode::REPORT_ALL_KEYS_AS_ESC, ~BindingMode::DISAMBIGUATE_ESC_CODES; Action::Esc("\x1bOS".into());
        Tab,       ModifiersState::SHIFT,   ~BindingMode::VI,   ~BindingMode::SEARCH, ~BindingMode::REPORT_ALL_KEYS_AS_ESC, ~BindingMode::DISAMBIGUATE_ESC_CODES; Action::Esc("\x1b[Z".into());
        Tab,       ModifiersState::SHIFT | ModifiersState::ALT, ~BindingMode::VI, ~BindingMode::SEARCH, ~BindingMode::REPORT_ALL_KEYS_AS_ESC, ~BindingMode::DISAMBIGUATE_ESC_CODES; Action::Esc("\x1b\x1b[Z".into());
        Backspace, ~BindingMode::VI, ~BindingMode::SEARCH, ~BindingMode::REPORT_ALL_KEYS_AS_ESC; Action::Esc("\x7f".into());
        Backspace, ModifiersState::ALT,     ~BindingMode::VI, ~BindingMode::SEARCH, ~BindingMode::REPORT_ALL_KEYS_AS_ESC, ~BindingMode::DISAMBIGUATE_ESC_CODES; Action::Esc("\x1b\x7f".into());
        Backspace, ModifiersState::SHIFT,   ~BindingMode::VI, ~BindingMode::SEARCH, ~BindingMode::REPORT_ALL_KEYS_AS_ESC, ~BindingMode::DISAMBIGUATE_ESC_CODES; Action::Esc("\x7f".into());
        Enter => KeyLocation::Numpad, ~BindingMode::VI, ~BindingMode::SEARCH, ~BindingMode::REPORT_ALL_KEYS_AS_ESC, ~BindingMode::DISAMBIGUATE_ESC_CODES; Action::Esc("\n".into());
        // Vi mode.
        " ", ModifiersState::SHIFT | ModifiersState::CONTROL, ~BindingMode::SEARCH; Action::ToggleViMode;
        " ", ModifiersState::SHIFT | ModifiersState::CONTROL, +BindingMode::VI, ~BindingMode::SEARCH; Action::ScrollToBottom;
        Escape,                             +BindingMode::VI, ~BindingMode::SEARCH; Action::ClearSelection;
        "i",                                +BindingMode::VI, ~BindingMode::SEARCH; Action::ToggleViMode;
        "i",                                +BindingMode::VI, ~BindingMode::SEARCH; Action::ScrollToBottom;
        "c",      ModifiersState::CONTROL,  +BindingMode::VI, ~BindingMode::SEARCH; Action::ToggleViMode;
        "y",      ModifiersState::CONTROL,  +BindingMode::VI, ~BindingMode::SEARCH; Action::ScrollLineUp;
        "e",      ModifiersState::CONTROL,  +BindingMode::VI, ~BindingMode::SEARCH; Action::ScrollLineDown;
        "g",                                +BindingMode::VI, ~BindingMode::SEARCH; Action::ScrollToTop;
        "g",      ModifiersState::SHIFT,    +BindingMode::VI, ~BindingMode::SEARCH; Action::ScrollToBottom;
        "b",      ModifiersState::CONTROL,  +BindingMode::VI, ~BindingMode::SEARCH; Action::ScrollPageUp;
        "f",      ModifiersState::CONTROL,  +BindingMode::VI, ~BindingMode::SEARCH; Action::ScrollPageDown;
        "u",      ModifiersState::CONTROL,  +BindingMode::VI, ~BindingMode::SEARCH; Action::ScrollHalfPageUp;
        "d",      ModifiersState::CONTROL,  +BindingMode::VI, ~BindingMode::SEARCH; Action::ScrollHalfPageDown;
        "y",                                +BindingMode::VI, ~BindingMode::SEARCH; Action::Copy;
        "y",                                +BindingMode::VI, ~BindingMode::SEARCH; Action::ClearSelection;
        "/",                                +BindingMode::VI, ~BindingMode::SEARCH; Action::SearchForward;
        "?",      ModifiersState::SHIFT,    +BindingMode::VI, ~BindingMode::SEARCH; Action::SearchBackward;
        "y",      ModifiersState::SHIFT,    +BindingMode::VI, ~BindingMode::SEARCH; ViAction::ToggleNormalSelection;
        "y",      ModifiersState::SHIFT,    +BindingMode::VI, ~BindingMode::SEARCH; ViMotion::Last;
        "y",      ModifiersState::SHIFT,    +BindingMode::VI, ~BindingMode::SEARCH; Action::Copy;
        "y",      ModifiersState::SHIFT,    +BindingMode::VI, ~BindingMode::SEARCH; Action::ClearSelection;
        "v",                                +BindingMode::VI, ~BindingMode::SEARCH; ViAction::ToggleNormalSelection;
        "v",      ModifiersState::SHIFT,    +BindingMode::VI, ~BindingMode::SEARCH; ViAction::ToggleLineSelection;
        "v",      ModifiersState::CONTROL,  +BindingMode::VI, ~BindingMode::SEARCH; ViAction::ToggleBlockSelection;
        "v",      ModifiersState::ALT,      +BindingMode::VI, ~BindingMode::SEARCH; ViAction::ToggleSemanticSelection;
        "n",                                +BindingMode::VI, ~BindingMode::SEARCH; ViAction::SearchNext;
        "n",      ModifiersState::SHIFT,    +BindingMode::VI, ~BindingMode::SEARCH; ViAction::SearchPrevious;
        Enter,                              +BindingMode::VI, ~BindingMode::SEARCH; ViAction::Open;
        "z",                                +BindingMode::VI, ~BindingMode::SEARCH; ViAction::CenterAroundViCursor;
        "f",                                +BindingMode::VI, ~BindingMode::SEARCH; ViAction::InlineSearchForward;
        "f",      ModifiersState::SHIFT,    +BindingMode::VI, ~BindingMode::SEARCH; ViAction::InlineSearchBackward;
        "t",                                +BindingMode::VI, ~BindingMode::SEARCH; ViAction::InlineSearchForwardShort;
        "t",      ModifiersState::SHIFT,    +BindingMode::VI, ~BindingMode::SEARCH; ViAction::InlineSearchBackwardShort;
        ";",                                +BindingMode::VI, ~BindingMode::SEARCH; ViAction::InlineSearchNext;
        ",",                                +BindingMode::VI, ~BindingMode::SEARCH; ViAction::InlineSearchPrevious;
        "*",      ModifiersState::SHIFT,    +BindingMode::VI, ~BindingMode::SEARCH; ViAction::SemanticSearchForward;
        "#",      ModifiersState::SHIFT,    +BindingMode::VI, ~BindingMode::SEARCH; ViAction::SemanticSearchBackward;
        "k",                                +BindingMode::VI, ~BindingMode::SEARCH; ViMotion::Up;
        "j",                                +BindingMode::VI, ~BindingMode::SEARCH; ViMotion::Down;
        "h",                                +BindingMode::VI, ~BindingMode::SEARCH; ViMotion::Left;
        "l",                                +BindingMode::VI, ~BindingMode::SEARCH; ViMotion::Right;
        ArrowUp,                            +BindingMode::VI, ~BindingMode::SEARCH; ViMotion::Up;
        ArrowDown,                          +BindingMode::VI, ~BindingMode::SEARCH; ViMotion::Down;
        ArrowLeft,                          +BindingMode::VI, ~BindingMode::SEARCH; ViMotion::Left;
        ArrowRight,                         +BindingMode::VI, ~BindingMode::SEARCH; ViMotion::Right;
        "0",                                +BindingMode::VI, ~BindingMode::SEARCH; ViMotion::First;
        "$",      ModifiersState::SHIFT,    +BindingMode::VI, ~BindingMode::SEARCH; ViMotion::Last;
        Home,                               +BindingMode::VI, ~BindingMode::SEARCH; ViMotion::First;
        End,                                +BindingMode::VI, ~BindingMode::SEARCH; ViMotion::Last;
        "^",      ModifiersState::SHIFT,    +BindingMode::VI, ~BindingMode::SEARCH; ViMotion::FirstOccupied;
        "h",      ModifiersState::SHIFT,    +BindingMode::VI, ~BindingMode::SEARCH; ViMotion::High;
        "m",      ModifiersState::SHIFT,    +BindingMode::VI, ~BindingMode::SEARCH; ViMotion::Middle;
        "l",      ModifiersState::SHIFT,    +BindingMode::VI, ~BindingMode::SEARCH; ViMotion::Low;
        "b",                                +BindingMode::VI, ~BindingMode::SEARCH; ViMotion::SemanticLeft;
        "w",                                +BindingMode::VI, ~BindingMode::SEARCH; ViMotion::SemanticRight;
        "e",                                +BindingMode::VI, ~BindingMode::SEARCH; ViMotion::SemanticRightEnd;
        "b",      ModifiersState::SHIFT,    +BindingMode::VI, ~BindingMode::SEARCH; ViMotion::WordLeft;
        "w",      ModifiersState::SHIFT,    +BindingMode::VI, ~BindingMode::SEARCH; ViMotion::WordRight;
        "e",      ModifiersState::SHIFT,    +BindingMode::VI, ~BindingMode::SEARCH; ViMotion::WordRightEnd;
        "%",      ModifiersState::SHIFT,    +BindingMode::VI, ~BindingMode::SEARCH; ViMotion::Bracket;
        "{",      ModifiersState::SHIFT,    +BindingMode::VI, ~BindingMode::SEARCH; ViMotion::ParagraphUp;
        "}",      ModifiersState::SHIFT,    +BindingMode::VI, ~BindingMode::SEARCH; ViMotion::ParagraphDown;
        Enter,                              +BindingMode::VI, +BindingMode::SEARCH; SearchAction::SearchConfirm;
        // Plain search.
        Escape,                             +BindingMode::SEARCH; SearchAction::SearchCancel;
        "c",      ModifiersState::CONTROL,  +BindingMode::SEARCH; SearchAction::SearchCancel;
        "u",      ModifiersState::CONTROL,  +BindingMode::SEARCH; SearchAction::SearchClear;
        "w",      ModifiersState::CONTROL,  +BindingMode::SEARCH; SearchAction::SearchDeleteWord;
        "p",      ModifiersState::CONTROL,  +BindingMode::SEARCH; SearchAction::SearchHistoryPrevious;
        "n",      ModifiersState::CONTROL,  +BindingMode::SEARCH; SearchAction::SearchHistoryNext;
        ArrowUp,                            +BindingMode::SEARCH; SearchAction::SearchHistoryPrevious;
        ArrowDown,                          +BindingMode::SEARCH; SearchAction::SearchHistoryNext;
        Enter,                              +BindingMode::SEARCH, ~BindingMode::VI; SearchAction::SearchFocusNext;
        Enter, ModifiersState::SHIFT,       +BindingMode::SEARCH, ~BindingMode::VI; SearchAction::SearchFocusPrevious;
    );

    bindings.extend(platform_key_bindings());

    bindings
}

#[cfg(not(any(target_os = "macos", test)))]
fn common_keybindings() -> Vec<KeyBinding> {
    bindings!(
        KeyBinding;
        "v",    ModifiersState::CONTROL | ModifiersState::SHIFT, ~BindingMode::VI;                       Action::Paste;
        "v",    ModifiersState::CONTROL | ModifiersState::SHIFT, +BindingMode::VI, +BindingMode::SEARCH; Action::Paste;
        "f",    ModifiersState::CONTROL | ModifiersState::SHIFT, ~BindingMode::SEARCH;                   Action::SearchForward;
        "b",    ModifiersState::CONTROL | ModifiersState::SHIFT, ~BindingMode::SEARCH;                   Action::SearchBackward;
        Insert, ModifiersState::SHIFT,                           ~BindingMode::VI;                       Action::PasteSelection;
        "c",    ModifiersState::CONTROL | ModifiersState::SHIFT;                                         Action::Copy;
        "c",    ModifiersState::CONTROL | ModifiersState::SHIFT, +BindingMode::VI, ~BindingMode::SEARCH; Action::ClearSelection;
        "0",    ModifiersState::CONTROL;                                                                 Action::ResetFontSize;
        "=",    ModifiersState::CONTROL;                                                                 Action::IncreaseFontSize;
        "+",    ModifiersState::CONTROL;                                                                 Action::IncreaseFontSize;
        "-",    ModifiersState::CONTROL;                                                                 Action::DecreaseFontSize;
        "+" => KeyLocation::Numpad, ModifiersState::CONTROL;                                             Action::IncreaseFontSize;
        "-" => KeyLocation::Numpad, ModifiersState::CONTROL;                                             Action::DecreaseFontSize;
    )
}

#[cfg(not(any(target_os = "macos", target_os = "windows", test)))]
pub fn platform_key_bindings() -> Vec<KeyBinding> {
    common_keybindings()
}

#[cfg(all(target_os = "windows", not(test)))]
pub fn platform_key_bindings() -> Vec<KeyBinding> {
    let mut bindings = bindings!(
        KeyBinding;
        Enter, ModifiersState::ALT; Action::ToggleFullscreen;
    );
    bindings.extend(common_keybindings());
    bindings
}

#[cfg(all(target_os = "macos", not(test)))]
pub fn platform_key_bindings() -> Vec<KeyBinding> {
    bindings!(
        KeyBinding;
        Insert, ModifiersState::SHIFT, ~BindingMode::VI, ~BindingMode::SEARCH; Action::Esc("\x1b[2;2~".into());
        // Tabbing api.
        "0",    ModifiersState::META;                                         Action::ResetFontSize;
        "=",    ModifiersState::META;                                         Action::IncreaseFontSize;
        "+",    ModifiersState::META;                                         Action::IncreaseFontSize;
        "-",    ModifiersState::META;                                         Action::DecreaseFontSize;
        "k",    ModifiersState::META, ~BindingMode::VI, ~BindingMode::SEARCH; Action::Esc("\x0c".into());
        "k",    ModifiersState::META, ~BindingMode::VI, ~BindingMode::SEARCH; Action::ClearHistory;
        "v",    ModifiersState::META, ~BindingMode::VI;                       Action::Paste;
        "v",    ModifiersState::META, +BindingMode::VI, +BindingMode::SEARCH; Action::Paste;
        "c",    ModifiersState::META;                                         Action::Copy;
        "c",    ModifiersState::META, +BindingMode::VI, ~BindingMode::SEARCH; Action::ClearSelection;
        // "n",    ModifiersState::META;                                         Action::CreateNewWindow;
        // "f",    ModifiersState::CONTROL | ModifiersState::META;               Action::ToggleFullscreen;
        "h",    ModifiersState::META;                                         Action::Window(WindowAction::Hide);
        // "h",    ModifiersState::META   | ModifiersState::ALT;                 Action::HideOtherApplications;
        "m",    ModifiersState::META;                                         Action::Window(WindowAction::Toggle);
        "q",    ModifiersState::META;                                         Action::Window(WindowAction::Quit);
        "w",    ModifiersState::META;                                         Action::Window(WindowAction::Quit);
        "f",    ModifiersState::META, ~BindingMode::SEARCH;                   Action::SearchForward;
        "b",    ModifiersState::META, ~BindingMode::SEARCH;                   Action::SearchBackward;
        "+" => KeyLocation::Numpad, ModifiersState::META;                     Action::IncreaseFontSize;
        "-" => KeyLocation::Numpad, ModifiersState::META;                     Action::DecreaseFontSize;
    )
}

// Don't return any bindings for tests since they are commented-out by default.
#[cfg(test)]
pub fn platform_key_bindings() -> Vec<KeyBinding> {
    vec![]
}
