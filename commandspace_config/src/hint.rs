use std::cell::{OnceCell, RefCell};
use std::fmt::{self, Formatter};
use std::mem;
use std::path::PathBuf;
use std::rc::Rc;

use log::{error, warn};
use serde::de::{Error as SerdeError, MapAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use unicode_width::UnicodeWidthChar;
use winit::keyboard::{Key, ModifiersState};

use crate::types::Program;
use alacritty_terminal::term::search::RegexSearch;

use super::LOG_TARGET_CONFIG;

use super::action::Action;
use super::bindings::{BindingKey, KeyBinding, KeyLocation, ModeWrapper, ModsWrapper};
/// Regex used for the default URL hint.
#[rustfmt::skip]
pub const URL_REGEX: &str = "(ipfs:|ipns:|magnet:|mailto:|gemini://|gopher://|https://|http://|news:|file:|git://|ssh:|ftp://)\
                         [^\u{0000}-\u{001F}\u{007F}-\u{009F}<>\"\\s{-}\\^⟨⟩`\\\\]+";

/// Regex terminal hints.
#[derive(serde::Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
#[serde(deny_unknown_fields, default)]
pub struct Hints {
    /// Characters for the hint labels.
    alphabet: HintsAlphabet,

    /// All configured terminal hints.
    pub enabled: Vec<Rc<Hint>>,
}

impl Default for Hints {
    fn default() -> Self {
        // Add URL hint by default when no other hint is present.
        let pattern = LazyRegexVariant::Pattern(String::from(URL_REGEX));
        let regex = LazyRegex(Rc::new(RefCell::new(pattern)));
        let content = HintContent::new(Some(regex), true);

        #[cfg(not(any(target_os = "macos", windows)))]
        let action = HintAction::Command(Program::Just(PathBuf::from("xdg-open")));
        #[cfg(target_os = "macos")]
        let action = HintAction::Command(Program::Just(PathBuf::from("open")));
        #[cfg(windows)]
        let action = HintAction::Command(Program::WithArgs {
            program: String::from("cmd"),
            args: vec!["/c".to_string(), "start".to_string(), "".to_string()],
        });

        Self {
            enabled: vec![Rc::new(Hint {
                content,
                action,
                persist: false,
                post_processing: true,
                mouse: Some(HintMouse { enabled: true, mods: Default::default() }),
                binding: Some(HintBinding {
                    key: BindingKey::Keycode {
                        key: Key::Character("o".into()),
                        location: KeyLocation::Standard,
                    },
                    mods: ModsWrapper(ModifiersState::SHIFT | ModifiersState::CONTROL),
                    cache: Default::default(),
                    mode: Default::default(),
                }),
            })],
            alphabet: Default::default(),
        }
    }
}

impl Hints {
    /// Characters for the hint labels.
    pub fn alphabet(&self) -> &str {
        &self.alphabet.0
    }
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct HintsAlphabet(String);

impl Default for HintsAlphabet {
    fn default() -> Self {
        Self(String::from("jfkdls;ahgurieowpq"))
    }
}

impl<'de> Deserialize<'de> for HintsAlphabet {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;

        let mut character_count = 0;
        for character in value.chars() {
            if character.width() != Some(1) {
                return Err(D::Error::custom("characters must be of width 1"));
            }
            character_count += 1;
        }

        if character_count < 2 {
            return Err(D::Error::custom("must include at last 2 characters"));
        }

        Ok(Self(value))
    }
}

/// Built-in actions for hint mode.
#[derive(serde::Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
pub enum HintInternalAction {
    /// Copy the text to the clipboard.
    Copy,
    /// Write the text to the PTY/search.
    Paste,
    /// Select the text matching the hint.
    Select,
    /// Move the vi mode cursor to the beginning of the hint.
    MoveViModeCursor,
}

/// Actions for hint bindings.
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
pub enum HintAction {
    /// Built-in hint action.
    #[serde(rename = "action")]
    Action(HintInternalAction),

    /// Command the text will be piped to.
    #[serde(rename = "command")]
    Command(Program),
}

/// Hint configuration.
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
pub struct Hint {
    /// Regex for finding matches.
    #[serde(flatten)]
    pub content: HintContent,

    /// Action executed when this hint is triggered.
    #[serde(flatten)]
    pub action: HintAction,

    /// Hint text post processing.
    #[serde(default)]
    pub post_processing: bool,

    /// Persist hints after selection.
    #[serde(default)]
    pub persist: bool,

    /// Hint mouse highlighting.
    pub mouse: Option<HintMouse>,

    /// Binding required to search for this hint.
    #[serde(skip_serializing)]
    pub binding: Option<HintBinding>,
}

#[derive(Serialize, Default, Clone, Debug, PartialEq, Eq)]
pub struct HintContent {
    /// Regex for finding matches.
    pub regex: Option<LazyRegex>,

    /// Escape sequence hyperlinks.
    pub hyperlinks: bool,
}

impl HintContent {
    pub fn new(regex: Option<LazyRegex>, hyperlinks: bool) -> Self {
        Self { regex, hyperlinks }
    }
}

impl<'de> Deserialize<'de> for HintContent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct HintContentVisitor;
        impl<'a> Visitor<'a> for HintContentVisitor {
            type Value = HintContent;

            fn expecting(&self, f: &mut Formatter<'_>) -> fmt::Result {
                f.write_str("a mapping")
            }

            fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
            where
                M: MapAccess<'a>,
            {
                let mut content = Self::Value::default();

                while let Some((key, value)) = map.next_entry::<String, toml::Value>()? {
                    match key.as_str() {
                        "regex" => match Option::<LazyRegex>::deserialize(value) {
                            Ok(regex) => content.regex = regex,
                            Err(err) => {
                                error!(target: LOG_TARGET_CONFIG, "Config error: hint's regex: {err}");
                            },
                        },
                        "hyperlinks" => match bool::deserialize(value) {
                            Ok(hyperlink) => content.hyperlinks = hyperlink,
                            Err(err) => {
                                error!(target: LOG_TARGET_CONFIG, "Config error: hint's hyperlinks: {err}");
                            },
                        },
                        "command" | "action" => (),
                        key => warn!(target: LOG_TARGET_CONFIG, "Unrecognized hint field: {key}"),
                    }
                }

                // Require at least one of hyperlinks or regex trigger hint matches.
                if content.regex.is_none() && !content.hyperlinks {
                    return Err(M::Error::custom(
                        "Config error: At least one of the hint's regex or hint's hyperlinks must \
                         be set",
                    ));
                }

                Ok(content)
            }
        }

        deserializer.deserialize_any(HintContentVisitor)
    }
}

/// Binding for triggering a keyboard hint.
#[derive(Deserialize, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct HintBinding {
    pub key: BindingKey,
    #[serde(default)]
    pub mods: ModsWrapper,
    #[serde(default)]
    pub mode: ModeWrapper,

    /// Cache for on-demand [`HintBinding`] to [`KeyBinding`] conversion.
    #[serde(skip)]
    cache: OnceCell<KeyBinding>,
}

impl HintBinding {
    /// Get the key binding for a hint.
    pub fn key_binding(&self, hint: &Rc<Hint>) -> &KeyBinding {
        self.cache.get_or_init(|| KeyBinding {
            trigger: self.key.clone(),
            mods: self.mods.0,
            mode: self.mode.mode,
            notmode: self.mode.not_mode,
            action: Action::Hint(hint.clone()),
        })
    }
}

impl fmt::Debug for HintBinding {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("HintBinding")
            .field("key", &self.key)
            .field("mods", &self.mods)
            .field("mode", &self.mode)
            .finish_non_exhaustive()
    }
}

/// Hint mouse highlighting.
#[derive(serde::Deserialize, Serialize, Default, Copy, Clone, Debug, PartialEq, Eq)]
pub struct HintMouse {
    /// Hint mouse highlighting availability.
    pub enabled: bool,

    /// Required mouse modifiers for hint highlighting.
    #[serde(skip_serializing)]
    pub mods: ModsWrapper,
}

/// Lazy regex with interior mutability.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LazyRegex(Rc<RefCell<LazyRegexVariant>>);

impl LazyRegex {
    /// Execute a function with the compiled regex DFAs as parameter.
    pub fn with_compiled<T, F>(&self, f: F) -> Option<T>
    where
        F: FnMut(&mut RegexSearch) -> T,
    {
        self.0.borrow_mut().compiled().map(f)
    }
}

impl<'de> Deserialize<'de> for LazyRegex {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let regex = LazyRegexVariant::Pattern(String::deserialize(deserializer)?);
        Ok(Self(Rc::new(RefCell::new(regex))))
    }
}

impl Serialize for LazyRegex {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let variant = self.0.borrow();
        let regex = match &*variant {
            LazyRegexVariant::Compiled(regex, _) => regex,
            LazyRegexVariant::Uncompilable(regex) => regex,
            LazyRegexVariant::Pattern(regex) => regex,
        };
        serializer.serialize_str(regex)
    }
}

/// Regex which is compiled on demand, to avoid expensive computations at startup.
#[derive(Clone, Debug)]
pub enum LazyRegexVariant {
    Compiled(String, Box<RegexSearch>),
    Pattern(String),
    Uncompilable(String),
}

impl LazyRegexVariant {
    /// Get a reference to the compiled regex.
    ///
    /// If the regex is not already compiled, this will compile the DFAs and store them for future
    /// access.
    fn compiled(&mut self) -> Option<&mut RegexSearch> {
        // Check if the regex has already been compiled.
        let regex = match self {
            Self::Compiled(_, regex_search) => return Some(regex_search),
            Self::Uncompilable(_) => return None,
            Self::Pattern(regex) => mem::take(regex),
        };

        // Compile the regex.
        let regex_search = match RegexSearch::new(&regex) {
            Ok(regex_search) => regex_search,
            Err(err) => {
                error!("could not compile hint regex: {err}");
                *self = Self::Uncompilable(regex);
                return None;
            },
        };
        *self = Self::Compiled(regex, Box::new(regex_search));

        // Return a reference to the compiled DFAs.
        match self {
            Self::Compiled(_, dfas) => Some(dfas),
            _ => unreachable!(),
        }
    }
}

impl PartialEq for LazyRegexVariant {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Pattern(regex), Self::Pattern(other_regex)) => regex == other_regex,
            _ => false,
        }
    }
}
impl Eq for LazyRegexVariant {}
