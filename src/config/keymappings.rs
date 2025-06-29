use std::{collections::HashMap, fmt::Display};

use itertools::Itertools;
use ratatui::crossterm::event::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct KeyMappings {
    #[serde(flatten)]
    pub bindings: HashMap<AppAction, Vec<KeyBinding>>,
}

impl KeyMappings {
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
        }
    }

    pub fn get(&self, action: AppAction) -> &Vec<KeyBinding> {
        self.bindings
            .get(&action)
            .expect("Key mapping not found, this indicates a bug and should not happen")
    }

    pub fn get_joined(&self, action: AppAction, sep: &str) -> String {
        self.get(action).iter().map(|b| b.to_string()).join(sep)
    }

    pub fn sorted(&self) -> std::vec::IntoIter<(&AppAction, &Vec<KeyBinding>)> {
        self.bindings.iter().sorted_by_key(|(key, _)| *key)
    }

    // test only
    pub fn insert(&mut self, action: AppAction, key_mappings: Vec<KeyBinding>) {
        self.bindings.insert(action, key_mappings);
    }

    pub fn override_with(mut self, key_mappings: KeyMappings) -> anyhow::Result<KeyMappings> {
        self.bindings.extend(key_mappings.bindings);
        self.validate_key_mappings()?;
        Ok(self)
    }

    pub fn preconfigured_mappings() -> KeyMappings {
        let default_config = r#"
next_item = ["down","tab", "ctrl+j", "ctrl+n"]
previous_item = ["up", "shift+tab", "ctrl+k", "ctrl+p"]
jump_ten_next_items = ["pagedown"]
jump_ten_previous_items = ["pageup"]
go_to_first_item = ["ctrl+up", "ctrl+home"]
go_to_last_item = ["ctrl+down", "ctrl+end"]

close = ["esc"]
quit = ["ctrl+c"]

kill_process = ["ctrl+x"]
refresh_process_list = ["ctrl+r"]
copy_process_pid = ["ctrl+y"]

scroll_process_details_down = ["ctrl+f"]
scroll_process_details_up = ["ctrl+b"]

select_process_parent = ["alt+p"]
select_process_family = ["alt+f"]
select_process_siblings = ["alt+s"]

toggle_help = ["ctrl+h"]
toggle_fps = ["ctrl+."]

cursor_left = ["left"]
cursor_right = ["right"]
cursor_home = ["home"]
cursor_end = ["end"]
delete_char = ["backspace"]
delete_next_char = ["delete"]
delete_word = ["ctrl+w"]
delete_to_start = ["ctrl+u"]
    "#;

        toml::from_str(default_config).expect("This should always be parseable")
    }

    fn validate_key_mappings(&self) -> anyhow::Result<()> {
        use crate::config::keymappings::{AppAction, KeyBinding};
        use ratatui::crossterm::event::{KeyCode, KeyModifiers};

        let mut used_bindings: HashMap<&KeyBinding, &AppAction> = HashMap::new();

        for (action, bindings) in self.bindings.iter() {
            for binding in bindings.iter() {
                // Validation 1: Disallow single character keys without modifiers
                if binding.modifier == KeyModifiers::NONE && matches!(binding.key, KeyCode::Char(_))
                {
                    anyhow::bail!(
                        "Key binding '{}' for action '{}' uses a single character without modifiers, which is generally disallowed.",
                        binding,
                        action
                    );
                }

                // Validation 2: Check for duplicate keybindings assigned to different actions
                if let Some(existing_action) = used_bindings.get(binding) {
                    if *existing_action != action {
                        anyhow::bail!(
                            "Duplicate key binding '{}' assigned to actions '{}' and '{}'.",
                            binding,
                            existing_action,
                            action
                        );
                    }
                } else {
                    used_bindings.insert(binding, action);
                }
            }
        }

        Ok(())
    }

    //TODO: consider using map<KeyBinding, AppAction> instead of HashMap<AppAction, Vec<KeyBinding>>
    pub fn resolve(&self, event: KeyEvent) -> AppAction {
        let looking_for = KeyBinding::from(event);

        self.bindings
            .iter()
            .find_map(|(action, bindings)| {
                if bindings.contains(&looking_for) {
                    Some(*action)
                } else {
                    None
                }
            })
            .unwrap_or(AppAction::Unmapped)
    }
}

//TODO: we should think about how to handle this better
//I don't like the fact that we user do not have config we will parse default config twice
impl Default for KeyMappings {
    fn default() -> Self {
        Self::preconfigured_mappings()
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord, Clone, Copy)]
#[serde(rename_all = "snake_case")]
// This order is reflected in help popup
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
    ToggleFps,

    CursorLeft,
    CursorRight,
    CursorHome,
    CursorEnd,
    DeleteChar,
    DeleteNextChar,
    DeleteWord,
    DeleteToStart,

    //Special case
    Unmapped,
}

impl Display for AppAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut snake_case = String::new();
        let serializer = toml::ser::ValueSerializer::new(&mut snake_case);
        self.serialize(serializer)
            .expect("failed to create snake_case for AppAction");
        f.write_str(&snake_case.replace("\"", ""))
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct KeyBinding {
    pub key: KeyCode,
    pub modifier: KeyModifiers,
}

impl From<KeyEvent> for KeyBinding {
    fn from(key_event: KeyEvent) -> Self {
        Self {
            key: key_event.code,
            modifier: key_event.modifiers,
        }
    }
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
        let modi = modifier_to_str(self.modifier);
        let sep = if modi.is_empty() { "" } else { MOD_SEPARATOR };
        let key = self.key.to_string().to_lowercase();
        write!(f, "{modi}{sep}{key}")
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
            return Err(format!("invalid modifier value '{invalid}'"));
        }
    };
    Ok(modif)
}

fn modifier_to_str(value: KeyModifiers) -> &'static str {
    match value {
        KeyModifiers::CONTROL => "ctrl",
        KeyModifiers::ALT => "alt",
        KeyModifiers::SHIFT => "shift",
        KeyModifiers::SUPER => "super",
        KeyModifiers::HYPER => "hyper",
        KeyModifiers::META => "meta",
        _ => "",
    }
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
            return Err(format!("invalid key value '{invalid}'"));
        }
    };
    Ok(key)
}

#[cfg(test)]
mod test {
    use std::collections::BTreeMap;

    use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    use crate::config::keymappings::{
        AppAction, KeyBinding, KeyMappings, modifier_to_str, str_to_key, str_to_modifier,
    };

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
    fn test_modifier_to_str() {
        assert_eq!(modifier_to_str(KeyModifiers::empty()), "");
        assert_eq!(modifier_to_str(KeyModifiers::CONTROL), "ctrl");
        assert_eq!(modifier_to_str(KeyModifiers::ALT), "alt");
        assert_eq!(modifier_to_str(KeyModifiers::SHIFT), "shift");
        assert_eq!(modifier_to_str(KeyModifiers::SUPER), "super");
        assert_eq!(modifier_to_str(KeyModifiers::META), "meta");
        assert_eq!(modifier_to_str(KeyModifiers::HYPER), "hyper");
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

    #[test]
    fn app_action_to_string_is_snake_case() {
        assert_eq!(AppAction::KillProcess.to_string(), "kill_process");
        assert_eq!(AppAction::Quit.to_string(), "quit");
        assert_eq!(
            AppAction::ScrollProcessDetailsDown.to_string(),
            "scroll_process_details_down"
        );
    }

    #[test]
    fn test_validate_key_mappings_that_are_valid() {
        let mut key_mappings = KeyMappings::new();
        key_mappings.insert(
            AppAction::Quit,
            vec![KeyBinding::char_with_mod('c', KeyModifiers::CONTROL)],
        );
        key_mappings.insert(
            AppAction::Close,
            vec![
                KeyBinding::key(KeyCode::Esc),
                KeyBinding::key(KeyCode::Enter),
            ],
        );
        key_mappings.insert(
            AppAction::NextItem,
            vec![
                KeyBinding::key(KeyCode::Down),
                KeyBinding::key(KeyCode::Tab),
            ],
        );

        let result = key_mappings.validate_key_mappings();
        assert!(
            result.is_ok(),
            "Validation should pass for valid key mappings"
        );
    }

    #[test]
    fn test_validate_key_mappings_single_char_no_modifier_fails() {
        let mut key_mappings = KeyMappings::new();
        // Invalid binding: 'a' without any modifier
        key_mappings.insert(
            AppAction::Quit,
            vec![KeyBinding::char_with_mod('a', KeyModifiers::NONE)],
        );

        let result = key_mappings.validate_key_mappings();
        assert!(
            result.is_err(),
            "Validation should fail for single character key without modifiers"
        );
        assert_eq!(
            result.unwrap_err().to_string(),
            "Key binding 'a' for action 'quit' uses a single character without modifiers, which is generally disallowed.",
        );
    }

    #[test]
    fn test_validate_key_mappings_duplicate_binding_different_action_fails() {
        let mut key_mappings = KeyMappings::new();
        let duplicate_binding = KeyBinding::char_with_mod('s', KeyModifiers::CONTROL);

        key_mappings.insert(AppAction::Quit, vec![duplicate_binding]);
        key_mappings.insert(AppAction::KillProcess, vec![duplicate_binding]); // Same binding for a different action

        let result = key_mappings.validate_key_mappings();
        assert!(
            result.is_err(),
            "Validation should fail for duplicate binding across different actions"
        );
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Duplicate key binding 'ctrl+s' assigned to actions"));
        assert!(err.contains("'quit'"));
        assert!(err.contains("'kill_process'"));
    }

    #[test]
    fn test_validate_key_mappings_duplicate_binding_same_action_ok() {
        let mut key_mappings = KeyMappings::new();
        let binding = KeyBinding::char_with_mod('c', KeyModifiers::CONTROL);

        // Same binding listed multiple times for the same action
        key_mappings.insert(AppAction::Quit, vec![binding, binding]);

        let result = key_mappings.validate_key_mappings();
        assert!(
            result.is_ok(),
            "Validation should pass for duplicate binding within the same action"
        );
    }

    #[test]
    fn test_override_preconfigured_keymappings() {
        let mut custom_key_mappings = KeyMappings::new();
        custom_key_mappings.insert(AppAction::Quit, vec![KeyBinding::key(KeyCode::PrintScreen)]);
        custom_key_mappings.insert(
            AppAction::KillProcess,
            vec![
                KeyBinding::key(KeyCode::Delete),
                KeyBinding::key_with_mod(KeyCode::Char('x'), KeyModifiers::SUPER),
                KeyBinding::key_with_mod(KeyCode::Char('x'), KeyModifiers::ALT),
            ],
        );
        custom_key_mappings.insert(
            AppAction::DeleteNextChar,
            vec![KeyBinding::key(KeyCode::CapsLock)],
        );

        let keymapping = KeyMappings::preconfigured_mappings()
            .override_with(custom_key_mappings)
            .expect("Should not fail");

        assert_eq!(
            keymapping.get(AppAction::Quit),
            &vec![KeyBinding::key(KeyCode::PrintScreen)],
            "Should override default key mapping for Quit"
        );
        assert_eq!(
            keymapping.get(AppAction::KillProcess),
            &vec![
                KeyBinding::key(KeyCode::Delete),
                KeyBinding::key_with_mod(KeyCode::Char('x'), KeyModifiers::SUPER),
                KeyBinding::key_with_mod(KeyCode::Char('x'), KeyModifiers::ALT),
            ],
            "Should override default key mapping for KillProcess"
        );
        assert_eq!(
            keymapping.get(AppAction::DeleteNextChar),
            &vec![KeyBinding::key(KeyCode::CapsLock)],
            "Should override default key mapping for DeleteNextChar"
        );
        // Validate that the default key mapping for DeleteNextChar is not present
        assert!(
            !keymapping
                .get(AppAction::DeleteNextChar)
                .contains(&KeyBinding::key(KeyCode::Delete)),
            "Should not contain default key mapping for DeleteNextChar"
        );
        // Validate other default key mappings were not overridden
        assert!(
            keymapping
                .get(AppAction::NextItem)
                .contains(&KeyBinding::key(KeyCode::Down)),
            "Should contain default key mapping for NextItem"
        );
        assert!(
            keymapping
                .get(AppAction::PreviousItem)
                .contains(&KeyBinding::key(KeyCode::Up)),
            "Should contain default key mapping for PreviousItem"
        );
    }

    #[test]
    fn test_resolve() {
        let mut key_mappings = KeyMappings::new();
        key_mappings.insert(
            AppAction::Quit,
            vec![
                KeyBinding::key(KeyCode::Esc),
                KeyBinding::char_with_mod('c', KeyModifiers::ALT),
            ],
        );
        key_mappings.insert(AppAction::NextItem, vec![KeyBinding::key(KeyCode::Down)]);
        key_mappings.insert(
            AppAction::ToggleHelp,
            vec![KeyBinding::char_with_mod('h', KeyModifiers::CONTROL)],
        );

        // Test mapped keys
        assert_eq!(
            key_mappings.resolve(KeyEvent::new(KeyCode::Esc, KeyModifiers::empty())),
            AppAction::Quit
        );
        assert_eq!(
            key_mappings.resolve(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::ALT)),
            AppAction::Quit
        );
        assert_eq!(
            key_mappings.resolve(KeyEvent::new(KeyCode::Down, KeyModifiers::empty())),
            AppAction::NextItem
        );
        assert_eq!(
            key_mappings.resolve(KeyEvent::new(KeyCode::Char('h'), KeyModifiers::CONTROL)),
            AppAction::ToggleHelp
        );

        // Test unmapped key
        assert_eq!(
            key_mappings.resolve(KeyEvent::new(KeyCode::Enter, KeyModifiers::empty())),
            AppAction::Unmapped
        );
        assert_eq!(
            key_mappings.resolve(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE)),
            AppAction::Unmapped
        );
        assert_eq!(
            key_mappings.resolve(KeyEvent::new(KeyCode::Up, KeyModifiers::SHIFT)),
            AppAction::Unmapped
        );
    }

    #[test]
    fn test_key_binding_from_key_event() {
        let key_event = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::empty());
        assert_eq!(
            KeyBinding::from(key_event),
            KeyBinding {
                key: KeyCode::Char('a'),
                modifier: KeyModifiers::empty(),
            }
        );

        let key_event = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        assert_eq!(
            KeyBinding::from(key_event),
            KeyBinding {
                key: KeyCode::Char('c'),
                modifier: KeyModifiers::CONTROL,
            }
        );

        let key_event = KeyEvent::new(KeyCode::Up, KeyModifiers::SHIFT);
        assert_eq!(
            KeyBinding::from(key_event),
            KeyBinding {
                key: KeyCode::Up,
                modifier: KeyModifiers::SHIFT,
            }
        );

        let key_event = KeyEvent::new(KeyCode::Delete, KeyModifiers::CONTROL | KeyModifiers::ALT);
        assert_eq!(
            KeyBinding::from(key_event),
            KeyBinding {
                key: KeyCode::Delete,
                modifier: KeyModifiers::CONTROL | KeyModifiers::ALT,
            }
        );
    }
}
