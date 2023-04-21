use std::{collections::VecDeque, iter::Iterator};

use chip8::KeyCode;
use serde::Deserialize;
use smol_str::SmolStr;
use winit::event::{ElementState, VirtualKeyCode};

/// Input mapper
///
/// Maps user input events to either Chip8 keycodes (suitable to be used in the VM),
/// or application specific named actions.
///
/// - *Chip8 Keycode*: These are the 16 keys of the old COSMAC VIP computer.
///   Stored in 8-bit integers and suitable to be passed to the virtual machine.
/// - *Named Action*: These are application specific input events that are
///   identified by a readable string.
#[derive(Debug)]
pub struct InputMap {
    actions: Box<[InputInfo]>,
    /// Mapping of host keyboard keys to application actions, by index.
    keys: Box<[(VirtualKeyCode, usize)]>,
    /// Buffer of collected events, as they happen.
    events: VecDeque<InputKind>,
    /// Current state of the key. Whether it is pressed down.
    state: Vec<InputState>,
}

#[derive(Debug)]
struct InputInfo {
    chip8: Option<KeyCode>,
    action: Option<SmolStr>,
    #[allow(dead_code)]
    keyboard_keys: Vec<VirtualKeyCode>,
}

/// Mapping to make optional fields infallible.
impl From<InputDef> for InputInfo {
    fn from(def: InputDef) -> Self {
        Self {
            chip8: def.chip8,
            action: def.action,
            keyboard_keys: def.keyboard_keys.unwrap_or_else(Vec::new),
        }
    }
}

#[derive(Debug, Deserialize)]
struct InputDef {
    chip8: Option<KeyCode>,
    action: Option<SmolStr>,
    keyboard_keys: Option<Vec<VirtualKeyCode>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputEvent {
    pub kind: InputKind,
    pub state: ElementState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputKind {
    Action(SmolStr),
    Chip8(u8),
}

impl InputKind {
    pub fn as_chip8(&self) -> Option<KeyCode> {
        match self {
            Self::Chip8(key_id) => match KeyCode::try_from(*key_id) {
                Ok(keycode) => Some(keycode),
                Err(_) => None,
            },
            _ => None,
        }
    }
}

#[derive(Debug)]
struct InputState {
    event: InputKind,
    pressed: bool,
}

impl InputMap {
    pub fn from_file(filepath: &str) -> std::io::Result<Self> {
        let mut file = std::fs::File::open(filepath)?;

        let defs: Vec<InputDef> = serde_yaml::from_reader(&mut file).unwrap();
        log::debug!("loaded input definitions: {:#?}", defs);

        let keys = Self::build_keys(&defs);
        let actions = defs.into_iter().map(InputInfo::from).collect();

        Ok(InputMap {
            actions,
            keys,
            events: VecDeque::new(),
            state: Vec::new(),
        })
    }

    /// Build a mapping of [`VirtualKeyCode`]s to indices into the given action definition mapping.
    fn build_keys(defs: &[InputDef]) -> Box<[(VirtualKeyCode, usize)]> {
        defs.iter()
            // definitions will be mapped by their index
            .enumerate()
            // lift keycodes out of the definitions
            .filter_map(|(index, def)| def.keyboard_keys.as_ref().map(|keys| (index, keys)))
            // flatten borrowed keycodes into one iterator of copied keycodes
            .flat_map(|(index, keys)| keys.iter().copied().map(move |keycode| (keycode, index)))
            .collect::<Vec<(VirtualKeyCode, usize)>>()
            .into_boxed_slice()
    }

    /// Given a user input keycode, map it to either a Chip8 key, or a named action.
    pub fn map_key(&self, key: VirtualKeyCode) -> Option<InputKind> {
        self.keys
            .iter()
            .find(|(keycode, _)| *keycode == key)
            .map(|(_, index)| *index)
            .and_then(|index| self.actions.get(index))
            .and_then(|input_def| {
                if let Some(key_code) = input_def.chip8 {
                    Some(InputKind::Chip8(key_code.as_u8()))
                } else if let Some(action_name) = &input_def.action {
                    Some(InputKind::Action(action_name.clone()))
                } else {
                    None
                }
            })
    }

    /// Push key event into the input state.
    pub fn push_key(&mut self, keycode: VirtualKeyCode, state: ElementState) {
        // Convert `winit` key to our input framework
        match self.map_key(keycode) {
            Some(event) => {
                // Stream of events in order
                self.events.push_back(event.clone());

                let pressed = state == ElementState::Pressed;

                // Map of state flags that can be checked by code
                match self.state.iter_mut().find(|el| el.event == event) {
                    Some(existing) => existing.pressed = pressed,
                    None => {
                        // Insert new state
                        self.state.push(InputState { event, pressed })
                    }
                }
            }
            None => {
                log::trace!("no input mapping for {keycode:?}");
            }
        }
    }

    pub fn is_action_pressed(&self, action: impl AsRef<str>) -> bool {
        let query = action.as_ref().trim();
        self.state
            .iter()
            .filter(|state| matches!(state.event, InputKind::Action(_)))
            .find(|state| match state.event {
                InputKind::Action(ref name) => name == query,
                _ => false,
            })
            .map(|state| state.pressed)
            .unwrap_or(false)
    }

    pub fn drain_events(&mut self) -> impl Iterator<Item = InputKind> + '_ {
        self.events.drain(..)
    }

    pub fn clear_state(&mut self) {
        // All keys that were release last frame must be removed
        self.state.retain(|state| state.pressed);

        for state in &mut self.state {
            state.pressed = false;
        }
    }

    pub fn iter_chip8(&self) -> impl Iterator<Item = KeyCode> + '_ {
        self.events.iter().filter_map(|ev| ev.as_chip8())
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_actions() {}
}
