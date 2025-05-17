use std::{collections::HashMap, fmt::Display};

use ratatui::crossterm::event::*;
use serde::Deserialize;

pub type KeyMappings = HashMap<AppAction, Vec<KeyBinding>>;

#[derive(Debug, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum AppAction {
    NextItem,
    PreviousItem,
    //TODO: consider jump half screen instead
    JumpTenNextItems,
    JumpTenPreviousItems,
    GoToFirstItem,
    GoToLastItem,

    Close,
    Quit,

    KillProcess,
    RefreshProcessList,
    CopyProcessPid,

    ScrollProcessDetailsDown,
    ScrollProcessDetailsUp,

    SelectProcessParent,
    SelectProcessFamily,
    SelectProcessSiblings,

    ToggleHelp,

    CursorLeft,
    CursorRight,
    CursorHome,
    CursorEnd,
    DeleteChar,
    DeleteNextChar,
    DeleteWord,
    DeleteToStart,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct KeyBinding {
    pub key: KeyCode,
    pub modifier: KeyModifiers,
}

impl KeyBinding {
    pub fn key_with_mod(key: KeyCode, modifier: KeyModifiers) -> Self {
        Self { key, modifier }
    }

    pub fn char_with_mod(key: char, modifier: KeyModifiers) -> Self {
        Self::key_with_mod(KeyCode::Char(key), modifier)
    }

    pub fn key(key: KeyCode) -> Self {
        Self {
            key,
            modifier: KeyModifiers::empty(),
        }
    }
}

impl Display for KeyBinding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let sep = if self.modifier.is_empty() {
            ""
        } else {
            MOD_SEPARATOR
        };
        let modi = self.modifier.to_string().to_lowercase();
        let key = self.key.to_string().to_lowercase();
        write!(f, "{}{}{}", modi, sep, key)
    }
}

const MOD_SEPARATOR: &str = "+";

impl<'de> Deserialize<'de> for KeyBinding {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value: String = Deserialize::deserialize(deserializer)?;
        let (modifier, key) = value
            .split_once(MOD_SEPARATOR)
            .unwrap_or_else(|| ("", &value));
        Ok(KeyBinding {
            key: str_to_key(key).map_err(serde::de::Error::custom)?,
            modifier: str_to_modifier(modifier).map_err(serde::de::Error::custom)?,
        })
    }
}

fn str_to_modifier(value: &str) -> Result<KeyModifiers, String> {
    let modif = match value {
        "" => KeyModifiers::empty(),
        "ctrl" => KeyModifiers::CONTROL,
        "alt" => KeyModifiers::ALT,
        "shift" => KeyModifiers::SHIFT,
        "super" => KeyModifiers::SUPER,
        "hyper" => KeyModifiers::HYPER,
        "meta" => KeyModifiers::META,
        invalid => {
            return Err(format!("invalid modifier value '{}'", invalid));
        }
    };
    Ok(modif)
}

fn str_to_key(value: &str) -> Result<KeyCode, String> {
    let key = match value {
        "esc" => KeyCode::Esc,
        "enter" => KeyCode::Enter,
        "tab" => KeyCode::Tab,
        "backspace" => KeyCode::Backspace,
        "up" => KeyCode::Up,
        "down" => KeyCode::Down,
        "left" => KeyCode::Left,
        "right" => KeyCode::Right,
        "space" => KeyCode::Char(' '),
        "pageup" => KeyCode::PageUp,
        "pagedown" => KeyCode::PageDown,
        "home" => KeyCode::Home,
        "end" => KeyCode::End,
        "insert" => KeyCode::Insert,
        "delete" => KeyCode::Delete,
        char if char.len() == 1 => KeyCode::Char(char.chars().next().unwrap()),
        invalid => {
            return Err(format!("invalid key value '{}'", invalid));
        }
    };
    Ok(key)
}

#[cfg(test)]
mod test {
    use std::collections::BTreeMap;

    use ratatui::crossterm::event::{KeyCode, KeyModifiers};

    use crate::config::keymappings::{AppAction, KeyBinding, str_to_key, str_to_modifier};

    #[test]
    fn test_deserialize_key_binding() {
        let mut expected = BTreeMap::new();
        expected.insert(
            AppAction::Close,
            vec![KeyBinding {
                key: KeyCode::Char('c'),
                modifier: KeyModifiers::CONTROL,
            }],
        );
        expected.insert(
            AppAction::Quit,
            vec![KeyBinding {
                key: KeyCode::Esc,
                modifier: KeyModifiers::empty(),
            }],
        );
        expected.insert(
            AppAction::KillProcess,
            vec![
                KeyBinding {
                    key: KeyCode::Char('z'),
                    modifier: KeyModifiers::ALT,
                },
                KeyBinding {
                    key: KeyCode::Char('z'),
                    modifier: KeyModifiers::CONTROL,
                },
            ],
        );
        let actual: BTreeMap<AppAction, Vec<KeyBinding>> = toml::from_str(
            r#"
quit = ["esc"]
close = ["ctrl+c"]
kill_process = ["alt+z", "ctrl+z"]
"#,
        )
        .unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_str_to_modifier() {
        assert_eq!(str_to_modifier(""), Ok(KeyModifiers::empty()));
        assert_eq!(str_to_modifier("ctrl"), Ok(KeyModifiers::CONTROL));
        assert_eq!(str_to_modifier("alt"), Ok(KeyModifiers::ALT));
        assert_eq!(str_to_modifier("shift"), Ok(KeyModifiers::SHIFT));
        assert_eq!(str_to_modifier("super"), Ok(KeyModifiers::SUPER));
        assert_eq!(str_to_modifier("meta"), Ok(KeyModifiers::META));
        assert_eq!(str_to_modifier("hyper"), Ok(KeyModifiers::HYPER));
        assert_eq!(
            str_to_modifier("invalid"),
            Err("invalid modifier value 'invalid'".to_string())
        );
        assert_eq!(
            str_to_modifier("ctrl+"),
            Err("invalid modifier value 'ctrl+'".to_string())
        );
    }

    #[test]
    fn test_str_to_key() {
        assert_eq!(str_to_key("esc"), Ok(KeyCode::Esc));
        assert_eq!(str_to_key("enter"), Ok(KeyCode::Enter));
        assert_eq!(str_to_key("tab"), Ok(KeyCode::Tab));
        assert_eq!(str_to_key("backspace"), Ok(KeyCode::Backspace));
        assert_eq!(str_to_key("up"), Ok(KeyCode::Up));
        assert_eq!(str_to_key("down"), Ok(KeyCode::Down));
        assert_eq!(str_to_key("left"), Ok(KeyCode::Left));
        assert_eq!(str_to_key("right"), Ok(KeyCode::Right));
        assert_eq!(str_to_key("space"), Ok(KeyCode::Char(' ')));
        assert_eq!(str_to_key("pageup"), Ok(KeyCode::PageUp));
        assert_eq!(str_to_key("pagedown"), Ok(KeyCode::PageDown));
        assert_eq!(str_to_key("home"), Ok(KeyCode::Home));
        assert_eq!(str_to_key("end"), Ok(KeyCode::End));
        assert_eq!(str_to_key("insert"), Ok(KeyCode::Insert));
        assert_eq!(str_to_key("delete"), Ok(KeyCode::Delete));
        assert_eq!(str_to_key("a"), Ok(KeyCode::Char('a')));
        assert_eq!(str_to_key("z"), Ok(KeyCode::Char('z')));
        assert_eq!(str_to_key("1"), Ok(KeyCode::Char('1')));
        assert_eq!(str_to_key("!"), Ok(KeyCode::Char('!')));
        assert_eq!(
            str_to_key("longkey"),
            Err("invalid key value 'longkey'".to_string())
        );
        assert_eq!(
            str_to_key("invalid+"),
            Err("invalid key value 'invalid+'".to_string())
        );
    }
}
