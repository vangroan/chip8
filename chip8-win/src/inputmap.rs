use std::collections::VecDeque;
use std::fmt;
use std::iter::Iterator;

use chip8::{Chip8Vm, KeyCode};
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
    actions: Box<[ActionInfo]>,
    /// Mapping of host keyboard keys to application actions, by index.
    keymap: Box<[(VirtualKeyCode, usize)]>,
    /// Mapping of action names to application actions, by index.
    namemap: Box<[(SmolStr, usize)]>,
    /// Buffer of collected events, as they happen.
    events: VecDeque<InputKind>,
    /// Current state of the key. Whether it is pressed down.
    state: Vec<InputState>,
}

#[derive(Debug)]
struct ActionInfo {
    chip8: Option<KeyCode>,
    action: Option<SmolStr>,
    #[allow(dead_code)]
    keyboard_keys: Vec<VirtualKeyCode>,
}

/// Mapping to make optional fields infallible.
impl From<InputDef> for ActionInfo {
    fn from(def: InputDef) -> Self {
        Self {
            chip8: def.chip8,
            action: def.action,
            keyboard_keys: def.keyboard_keys.unwrap_or_default(),
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
pub struct InputState {
    pub kind: InputKind,
    pub key_state: KeyState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyState {
    Pressed,
    Released,
    Down,
}

impl KeyState {
    pub fn is_pressed(&self) -> bool {
        *self == Self::Pressed
    }

    pub fn is_released(&self) -> bool {
        *self == Self::Released
    }

    pub fn is_down(&self) -> bool {
        matches!(self, Self::Pressed | Self::Down)
    }
}

impl From<winit::event::ElementState> for KeyState {
    fn from(key_state: winit::event::ElementState) -> Self {
        match key_state {
            winit::event::ElementState::Pressed => KeyState::Pressed,
            winit::event::ElementState::Released => KeyState::Released,
        }
    }
}

impl fmt::Display for KeyState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let name = match self {
            Self::Pressed => "pressed",
            Self::Released => "released",
            Self::Down => "down",
        };
        write!(f, "{name}")
    }
}

impl InputMap {
    /// Load an input map from a YAML file.
    pub fn from_file(filepath: &str) -> std::io::Result<Self> {
        let mut file = std::fs::File::open(filepath)?;

        let defs: Vec<InputDef> = serde_yaml::from_reader(&mut file).unwrap();
        log::debug!("loaded input definitions: {:#?}", defs);

        let mut inputmap = InputMap {
            actions: defs.into_iter().map(ActionInfo::from).collect(),
            keymap: Box::new([]),
            namemap: Box::new([]),
            events: VecDeque::new(),
            state: Vec::new(),
        };

        inputmap.rebuild_mappings();

        Ok(inputmap)
    }

    /// Rebuild the input mappings to actions,
    /// for when the actions have been changed.
    fn rebuild_mappings(&mut self) {
        let mut keymap = vec![];
        let mut namemap = vec![];

        self.actions.iter().enumerate().for_each(|(index, action)| {
            for key in &action.keyboard_keys {
                keymap.push((*key, index));
            }

            if let Some(ref name) = action.action {
                namemap.push((name.clone(), index));
            }
        });

        self.keymap = keymap.into_boxed_slice();
        self.namemap = namemap.into_boxed_slice();
    }

    /// Given a user input keycode, map it to either a Chip8 key, or a named action.
    pub fn map_key(&self, key: VirtualKeyCode) -> Option<InputKind> {
        self.keymap
            .iter()
            .find(|(keycode, _)| *keycode == key)
            .map(|(_, index)| *index)
            .and_then(|index| self.actions.get(index))
            .and_then(|input_def| {
                if input_def.chip8.is_some() {
                    input_def
                        .chip8
                        .map(|key_code| key_code.as_u8())
                        .map(InputKind::Chip8)
                } else if input_def.action.is_some() {
                    input_def.action.clone().map(InputKind::Action)
                } else {
                    None
                }
            })
    }

    /// Process the internal input state.
    ///
    /// Call this at a frame boundary to prepare for new events.
    pub fn process(&mut self) {
        // clear out released inputs.
        self.clear_releases();

        // Advance pressed state to down.
        self.state
            .iter_mut()
            .filter(|ev| ev.key_state.is_pressed())
            .for_each(|ev| ev.key_state = KeyState::Down);

        // Clear event queue
        for _ in self.drain_events() {}
    }

    fn set_state(&mut self, kind: InputKind, key_state: KeyState) {
        match self.state.iter_mut().find(|el| el.kind == kind) {
            Some(existing) => existing.key_state = key_state,
            None => {
                // Insert new state
                self.state.push(InputState { kind, key_state })
            }
        }
    }

    /// Emit a key event.
    pub fn emit_key(&mut self, keycode: VirtualKeyCode, element_state: ElementState) {
        // Convert `winit` key to our input framework
        match self.map_key(keycode) {
            Some(kind) => {
                // Stream of events in order
                self.events.push_back(kind.clone());
                self.set_state(kind, KeyState::from(element_state));
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
            .filter(|state| matches!(state.kind, InputKind::Action(_)))
            .find(|state| match state.kind {
                InputKind::Action(ref name) => name == query,
                _ => false,
            })
            .map(|state| state.key_state.is_down())
            .unwrap_or(false)
    }

    pub fn is_action_released(&self, action: impl AsRef<str>) -> bool {
        self.action_state(action)
            .map(|state| state.key_state.is_released())
            .unwrap_or(false)
    }

    pub fn action_state(&self, action: impl AsRef<str>) -> Option<&InputState> {
        let query = action.as_ref().trim();
        self.state
            .iter()
            .filter(|state| matches!(state.kind, InputKind::Action(_)))
            .find(|state| match state.kind {
                InputKind::Action(ref name) => name == query,
                _ => false,
            })
    }

    /// Remove all queued events.
    pub fn drain_events(&mut self) -> impl Iterator<Item = InputKind> + '_ {
        self.events.drain(..)
    }

    /// Discard all key release states.
    pub fn clear_releases(&mut self) {
        // All keys that were release last frame must be removed
        self.state.retain(|state| state.key_state.is_down());
    }

    /// Return all Chip8 keys that are down.
    pub fn iter_chip8(&self) -> impl Iterator<Item = KeyCode> + '_ {
        self.state
            .iter()
            .filter(|ev| ev.key_state.is_down())
            .filter_map(|ev| ev.kind.as_chip8())
    }

    // Write keyboard input into Chip8 VM.
    pub fn write_keys(&mut self, vm: &mut Chip8Vm) {
        vm.clear_keys();
        for keycode in self.iter_chip8() {
            vm.set_key(keycode, true);
        }
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_actions() {}
}
