//! User configuration, stored as TOML in the platform config directory.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Buttons that can switch, by HID++ control ID (REPROG_CONTROLS_V4).
pub const BUTTON_WHEEL_MODE: u16 = 0x00C4;
pub const BUTTON_GESTURE: u16 = 0x00C3;
pub const BUTTON_MIDDLE: u16 = 0x0052;
pub const BUTTONS: [u16; 3] = [BUTTON_WHEEL_MODE, BUTTON_GESTURE, BUTTON_MIDDLE];

/// Gesture: hold the button, move the mouse, let go.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gesture {
    /// Pressed without moving.
    Press,
    Up,
    Down,
    Left,
    Right,
}

impl Gesture {
    /// Mouse movement (raw counts) below this while the button is held is a press.
    const MIN_MOVE: i32 = 50;

    /// Classifies the movement made while the button was held (y grows downwards).
    pub fn from_movement(dx: i32, dy: i32) -> Gesture {
        if dx.abs().max(dy.abs()) < Self::MIN_MOVE {
            Gesture::Press
        } else if dx.abs() >= dy.abs() {
            if dx > 0 { Gesture::Right } else { Gesture::Left }
        } else if dy > 0 {
            Gesture::Down
        } else {
            Gesture::Up
        }
    }
}

/// Where one hold-and-move gesture switches to: its own channels.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct MoveSetting {
    pub on: bool,
    pub channels: Vec<u8>,
}

impl Default for MoveSetting {
    fn default() -> Self {
        MoveSetting { on: false, channels: vec![1, 2, 3] }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Moves {
    pub up: MoveSetting,
    pub down: MoveSetting,
    pub left: MoveSetting,
    pub right: MoveSetting,
}

impl Moves {
    pub fn is_empty(&self) -> bool {
        *self == Moves::default()
    }

    /// Whether any direction is switched on.
    pub fn any_on(&self) -> bool {
        [&self.up, &self.down, &self.left, &self.right].iter().any(|m| m.on)
    }

    pub fn get(&self, g: Gesture) -> Option<&MoveSetting> {
        match g {
            Gesture::Up => Some(&self.up),
            Gesture::Down => Some(&self.down),
            Gesture::Left => Some(&self.left),
            Gesture::Right => Some(&self.right),
            Gesture::Press => None,
        }
    }

    pub fn get_mut(&mut self, g: Gesture) -> Option<&mut MoveSetting> {
        match g {
            Gesture::Up => Some(&mut self.up),
            Gesture::Down => Some(&mut self.down),
            Gesture::Left => Some(&mut self.left),
            Gesture::Right => Some(&mut self.right),
            Gesture::Press => None,
        }
    }
}

/// The next of `channels` after `current` (both 1-based), among the device's `count`,
/// like the Easy-Switch button under the mouse: with two channels it goes back and
/// forth, with one it always goes there. None when it is already there.
pub fn pick(channels: &[u8], current: u8, count: u8) -> Option<u8> {
    let usable: Vec<u8> = channels.iter().copied().filter(|&c| c <= count).collect();
    let next = match usable.iter().position(|&c| c == current) {
        Some(i) => usable[(i + 1) % usable.len()],
        None => *usable.first()?,
    };
    (next != current).then_some(next)
}

/// What one button does.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ButtonSetting {
    /// Whether this button switches at all.
    pub on: bool,
    /// Channels (1-based) a press (without moving the mouse) steps through.
    pub channels: Vec<u8>,
    /// Hold-and-move gestures (used for the gesture button). While any is set, the
    /// mouse movement during a hold goes to flockswitch, so the cursor stays still.
    #[serde(skip_serializing_if = "Moves::is_empty")]
    pub moves: Moves,
}

impl Default for ButtonSetting {
    fn default() -> Self {
        ButtonSetting { on: false, channels: vec![1, 2, 3], moves: Moves::default() }
    }
}

impl ButtonSetting {
    /// The channel (1-based) a gesture leads to from the current one (0-based), given
    /// how many channels the device has; None when nothing should happen (already
    /// there, or that direction is off).
    pub fn channel_after(&self, gesture: Gesture, current0: u8, count: u8) -> Option<u8> {
        let current = current0 + 1;
        match self.moves.get(gesture) {
            None => pick(&self.channels, current, count),
            Some(m) if m.on => pick(&m.channels, current, count),
            Some(_) => None,
        }
    }

    /// Whether the button needs the mouse movement during a hold.
    pub fn uses_moves(&self) -> bool {
        self.moves.any_on()
    }
}

/// Each button that can switch has its own setting.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Buttons {
    /// Wheel mode-shift button (control 0x00C4).
    pub wheel_mode: ButtonSetting,
    /// Gesture (thumb) button (0x00C3).
    pub gesture: ButtonSetting,
    /// Middle button, the wheel click (0x0052).
    pub middle: ButtonSetting,
}

impl Default for Buttons {
    fn default() -> Self {
        Buttons {
            wheel_mode: ButtonSetting { on: true, ..ButtonSetting::default() },
            gesture: ButtonSetting::default(),
            middle: ButtonSetting::default(),
        }
    }
}

/// A side of the screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Edge {
    Left,
    Right,
    Top,
    Bottom,
}

pub const EDGES: [Edge; 4] = [Edge::Left, Edge::Right, Edge::Top, Edge::Bottom];

impl Edge {
    /// Name in the settings file and on the command line.
    pub fn name(self) -> &'static str {
        match self {
            Edge::Left => "left",
            Edge::Right => "right",
            Edge::Top => "top",
            Edge::Bottom => "bottom",
        }
    }

    pub fn from_name(name: &str) -> Option<Edge> {
        EDGES.into_iter().find(|e| e.name() == name.trim().to_lowercase())
    }
}

/// The colour of the on-screen effects until another is chosen.
pub const DEFAULT_EFFECT_COLOR: &str = "#FFCC00";

/// Red, green and blue of a colour written as "#RRGGBB".
pub fn parse_color(text: &str) -> Option<[u8; 3]> {
    let hex = text.trim().strip_prefix('#')?;
    if hex.len() != 6 || !hex.is_ascii() {
        return None;
    }
    let byte = |at: usize| u8::from_str_radix(&hex[at..at + 2], 16).ok();
    Some([byte(0)?, byte(2)?, byte(4)?])
}

/// The sizes the settings window offers, in percent; the command line takes 50-200.
pub const EFFECT_SIZES: [u16; 4] = [75, 100, 125, 150];

/// "125" or "125%".
fn percent(text: &str) -> Option<u16> {
    text.trim().trim_end_matches('%').parse().ok()
}

/// A colour as the settings write it.
pub fn format_color([r, g, b]: [u8; 3]) -> String {
    format!("#{r:02X}{g:02X}{b:02X}")
}

/// What is drawn near the cursor while it waits at a screen edge. The number of each
/// variant indexes the table of drawing functions in overlay_render.rs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EdgeAnimStyle {
    None,
    #[default]
    Ring,
    Semicircle,
    Bar,
    Pulse,
    Laser,
    Vortex,
    Scan,
    Liquid,
    /// A different effect each time; see `resolved`. Has no drawing function of its own.
    Random,
}

pub const EDGE_ANIM_STYLES: [EdgeAnimStyle; 10] = [
    EdgeAnimStyle::None,
    EdgeAnimStyle::Ring,
    EdgeAnimStyle::Semicircle,
    EdgeAnimStyle::Bar,
    EdgeAnimStyle::Pulse,
    EdgeAnimStyle::Laser,
    EdgeAnimStyle::Vortex,
    EdgeAnimStyle::Scan,
    EdgeAnimStyle::Liquid,
    EdgeAnimStyle::Random,
];

/// A number below `n`, different from run to run. Good enough to choose an effect.
pub(crate) fn random_below(n: usize) -> usize {
    use std::hash::{BuildHasher, Hasher};
    std::collections::hash_map::RandomState::new().build_hasher().finish() as usize % n
}

impl EdgeAnimStyle {
    /// Name in the settings file and on the command line.
    pub fn name(self) -> &'static str {
        match self {
            EdgeAnimStyle::None => "none",
            EdgeAnimStyle::Ring => "ring",
            EdgeAnimStyle::Semicircle => "semicircle",
            EdgeAnimStyle::Bar => "bar",
            EdgeAnimStyle::Pulse => "pulse",
            EdgeAnimStyle::Laser => "laser",
            EdgeAnimStyle::Vortex => "vortex",
            EdgeAnimStyle::Scan => "scan",
            EdgeAnimStyle::Liquid => "liquid",
            EdgeAnimStyle::Random => "random",
        }
    }

    fn from_name(name: &str) -> Option<EdgeAnimStyle> {
        EDGE_ANIM_STYLES.into_iter().find(|s| s.name() == name.trim().to_lowercase())
    }

    /// The effect to draw now: for Random one of the others (never None).
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn resolved(self) -> EdgeAnimStyle {
        match self {
            EdgeAnimStyle::Random => EDGE_EFFECTS[random_below(EDGE_EFFECTS.len())],
            style => style,
        }
    }

    /// Where its own settings are in `Edges::effects`; None for None and Random.
    fn effect_index(self) -> Option<usize> {
        EDGE_EFFECTS.iter().position(|&s| s == self)
    }

    /// Whether the effect drains in a direction the user can choose.
    pub fn has_direction(self) -> bool {
        matches!(self, EdgeAnimStyle::Ring | EdgeAnimStyle::Semicircle | EdgeAnimStyle::Bar)
    }

    /// Whether the effect can shrink away instead of draining.
    pub fn can_shrink(self) -> bool {
        matches!(self, EdgeAnimStyle::Ring | EdgeAnimStyle::Semicircle)
    }
}

/// Which way the ring, semicircle or bar drains. Numbered as in overlay_render.rs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EdgeAnimDirection {
    #[default]
    Clockwise,
    CounterClockwise,
    Both,
}

pub const EDGE_ANIM_DIRECTIONS: [EdgeAnimDirection; 3] =
    [EdgeAnimDirection::Clockwise, EdgeAnimDirection::CounterClockwise, EdgeAnimDirection::Both];

impl EdgeAnimDirection {
    fn from_name(name: &str) -> Option<EdgeAnimDirection> {
        match name.trim().to_lowercase().as_str() {
            "clockwise" => Some(EdgeAnimDirection::Clockwise),
            "counterclockwise" => Some(EdgeAnimDirection::CounterClockwise),
            "both" => Some(EdgeAnimDirection::Both),
            _ => None,
        }
    }
}

/// The effects that draw something, each with settings of its own, in the order of
/// `EdgeAnimStyle` (and of the `[edges.effects.<name>]` tables in the settings file).
pub const EDGE_EFFECTS: [EdgeAnimStyle; 8] = [
    EdgeAnimStyle::Ring,
    EdgeAnimStyle::Semicircle,
    EdgeAnimStyle::Bar,
    EdgeAnimStyle::Pulse,
    EdgeAnimStyle::Laser,
    EdgeAnimStyle::Vortex,
    EdgeAnimStyle::Scan,
    EdgeAnimStyle::Liquid,
];

/// How one countdown effect looks: `[edges.effects.<name>]` in the settings file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectSettings {
    /// "#RRGGBB".
    pub color: String,
    /// Size in percent.
    pub size: u16,
    /// Which way it drains; only for effects with `has_direction` (others keep the default).
    pub direction: EdgeAnimDirection,
    /// Shrinks away instead of draining; only for effects that `can_shrink`.
    pub shrink: bool,
}

impl Default for EffectSettings {
    fn default() -> Self {
        EffectSettings { color: DEFAULT_EFFECT_COLOR.into(), size: 100, direction: EdgeAnimDirection::default(), shrink: false }
    }
}

/// What one countdown draws: the effect (Random already picked) and how.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EdgeLook {
    pub style: EdgeAnimStyle,
    pub direction: EdgeAnimDirection,
    pub shrink: bool,
    pub color: String,
    pub size: u16,
}

/// How the cursor is shown. The number of each variant indexes the table of drawing
/// functions in overlay_render.rs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LocateStyle {
    #[default]
    Ring,
    Radar,
    Spotlight,
    /// A different effect each time; see `resolved`. Has no drawing function of its own.
    Random,
}

impl LocateStyle {
    /// The effect to draw now: for Random one of the others.
    pub fn resolved(self) -> LocateStyle {
        match self {
            LocateStyle::Random => [LocateStyle::Ring, LocateStyle::Radar, LocateStyle::Spotlight][random_below(3)],
            style => style,
        }
    }
}

/// A key that must be held while the mouse is shaken, for the shake to count.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ShakeKey {
    #[default]
    None,
    Ctrl,
    Alt,
    Shift,
}

/// The shake times the settings window offers, in milliseconds.
pub const SHAKE_TIMES: [u64; 4] = [500, 1000, 1500, 2000];

impl ShakeKey {
    /// Name in the settings file and on the command line.
    pub fn name(self) -> &'static str {
        match self {
            ShakeKey::None => "none",
            ShakeKey::Ctrl => "ctrl",
            ShakeKey::Alt => "alt",
            ShakeKey::Shift => "shift",
        }
    }

    /// Also takes the names other systems use for the same key ("control", "option").
    pub fn from_name(name: &str) -> Option<ShakeKey> {
        match name.trim().to_lowercase().as_str() {
            "none" | "" => Some(ShakeKey::None),
            "ctrl" | "control" => Some(ShakeKey::Ctrl),
            "alt" | "option" => Some(ShakeKey::Alt),
            "shift" => Some(ShakeKey::Shift),
            // The macOS version offers ⌘; Windows has no key for a shake there.
            "cmd" | "command" | "super" | "meta" | "win" => Some(ShakeKey::None),
            _ => None,
        }
    }
}

impl Serialize for ShakeKey {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(self.name())
    }
}

/// A key this version does not know (say, one only another system has) reads as none
/// instead of making the whole settings file unreadable.
impl<'de> Deserialize<'de> for ShakeKey {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<ShakeKey, D::Error> {
        Ok(ShakeKey::from_name(&String::deserialize(d)?).unwrap_or_default())
    }
}

/// Showing where the cursor is with a short highlight around it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Locate {
    pub on: bool,
    /// Shown when the shortcut below is pressed.
    pub on_hotkey: bool,
    /// e.g. "ctrl+alt+l"; empty for none.
    pub hotkey: String,
    /// Shown when the mouse arrives from another computer.
    pub on_arrival: bool,
    /// Shown when the mouse is shaken from side to side.
    pub on_shake: bool,
    /// How long (ms) the shaking must go on, back and forth without a break.
    pub shake_time_ms: u64,
    /// A key that must be held while shaking; none for any shake.
    pub shake_key: ShakeKey,
    pub style: LocateStyle,
    /// "#RRGGBB".
    pub color: String,
    /// Size of the highlight in percent.
    pub size: u16,
}

impl Default for Locate {
    fn default() -> Self {
        Locate {
            on: false,
            on_hotkey: true,
            hotkey: "ctrl+alt+l".into(),
            on_arrival: true,
            on_shake: false,
            shake_time_ms: 1000,
            shake_key: ShakeKey::None,
            style: LocateStyle::default(),
            color: DEFAULT_EFFECT_COLOR.into(),
            size: 100,
        }
    }
}

/// Switching at the screen edge: rest the cursor against an outer edge of the desktop
/// and the group goes to that edge's channel. Each computer watches its own screen.
///
/// Each countdown effect has its own colour, size, direction and shrinking, written as
/// `[edges.effects.<name>]`. Versions before 1.8.0 had one `anim_color`, `anim_size`,
/// `anim_direction` and `anim_shrink` for every effect: those keys are still written,
/// mirroring the chosen effect, so older versions read the file as before; reading a file
/// without the tables gives every effect those values.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Edges {
    pub on: bool,
    /// How long (ms) the cursor must rest against the edge first.
    pub dwell_ms: u64,
    /// Channel (1-3) each edge goes to; 0 = that edge does nothing.
    pub left: u8,
    pub right: u8,
    pub top: u8,
    pub bottom: u8,
    /// Pixels at each end of an edge that do not count (hot corners, window buttons).
    pub corner_margin: u32,
    /// When the mouse comes back, put the cursor at the edge it left by.
    pub place_cursor: bool,
    /// The wait runs only while Control is held; letting go calls it off.
    pub require_control: bool,
    /// What is drawn while the cursor waits.
    pub anim_style: EdgeAnimStyle,
    /// The settings of each effect, in the order of `EDGE_EFFECTS`.
    pub effects: [EffectSettings; 8],
}

impl Default for Edges {
    fn default() -> Self {
        Edges {
            on: false,
            dwell_ms: 500,
            left: 0,
            right: 0,
            top: 0,
            bottom: 0,
            corner_margin: 80,
            place_cursor: true,
            require_control: false,
            anim_style: EdgeAnimStyle::default(),
            effects: Default::default(),
        }
    }
}

/// `[edges]` as written: the legacy keys mirror the chosen effect, the tables come last.
#[derive(Serialize)]
struct EdgesOut<'a> {
    on: bool,
    dwell_ms: u64,
    left: u8,
    right: u8,
    top: u8,
    bottom: u8,
    corner_margin: u32,
    place_cursor: bool,
    require_control: bool,
    anim_style: EdgeAnimStyle,
    anim_direction: EdgeAnimDirection,
    anim_shrink: bool,
    anim_color: &'a str,
    anim_size: u16,
    effects: EffectsOut<'a>,
}

struct EffectsOut<'a>(&'a [EffectSettings; 8]);

#[derive(Serialize)]
struct EffectOut<'a> {
    color: &'a str,
    size: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    direction: Option<EdgeAnimDirection>,
    #[serde(skip_serializing_if = "Option::is_none")]
    shrink: Option<bool>,
}

impl Serialize for EffectsOut<'_> {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap;
        let mut map = s.serialize_map(Some(EDGE_EFFECTS.len()))?;
        for (style, e) in EDGE_EFFECTS.into_iter().zip(self.0) {
            let out = EffectOut {
                color: &e.color,
                size: e.size,
                direction: style.has_direction().then_some(e.direction),
                shrink: style.can_shrink().then_some(e.shrink),
            };
            map.serialize_entry(style.name(), &out)?;
        }
        map.end()
    }
}

impl Serialize for Edges {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mirrored = self.mirrored();
        EdgesOut {
            on: self.on,
            dwell_ms: self.dwell_ms,
            left: self.left,
            right: self.right,
            top: self.top,
            bottom: self.bottom,
            corner_margin: self.corner_margin,
            place_cursor: self.place_cursor,
            require_control: self.require_control,
            anim_style: self.anim_style,
            anim_direction: mirrored.direction,
            anim_shrink: mirrored.shrink,
            anim_color: &mirrored.color,
            anim_size: mirrored.size,
            effects: EffectsOut(&self.effects),
        }
        .serialize(s)
    }
}

/// `[edges]` as read, with the legacy keys that fill in what an effect's table lacks.
#[derive(Deserialize)]
#[serde(default)]
struct EdgesIn {
    on: bool,
    dwell_ms: u64,
    left: u8,
    right: u8,
    top: u8,
    bottom: u8,
    corner_margin: u32,
    place_cursor: bool,
    require_control: bool,
    anim_style: EdgeAnimStyle,
    anim_direction: Option<EdgeAnimDirection>,
    anim_shrink: Option<bool>,
    anim_color: Option<String>,
    anim_size: Option<u16>,
    /// Any value: a table of a name this version does not know is passed over.
    effects: std::collections::BTreeMap<String, toml::Value>,
}

impl Default for EdgesIn {
    fn default() -> Self {
        let d = Edges::default();
        EdgesIn {
            on: d.on,
            dwell_ms: d.dwell_ms,
            left: d.left,
            right: d.right,
            top: d.top,
            bottom: d.bottom,
            corner_margin: d.corner_margin,
            place_cursor: d.place_cursor,
            require_control: d.require_control,
            anim_style: d.anim_style,
            anim_direction: None,
            anim_shrink: None,
            anim_color: None,
            anim_size: None,
            effects: Default::default(),
        }
    }
}

/// One `[edges.effects.<name>]`; fields it does not know are passed over.
#[derive(Deserialize, Default)]
#[serde(default)]
struct EffectIn {
    color: Option<String>,
    size: Option<u16>,
    direction: Option<EdgeAnimDirection>,
    shrink: Option<bool>,
}

impl<'de> Deserialize<'de> for Edges {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Edges, D::Error> {
        let file = EdgesIn::deserialize(d)?;
        let mut effects: [EffectSettings; 8] = Default::default();
        for (style, e) in EDGE_EFFECTS.into_iter().zip(effects.iter_mut()) {
            let own: EffectIn = match file.effects.get(style.name()) {
                Some(value) => value
                    .clone()
                    .try_into()
                    .map_err(|err| serde::de::Error::custom(format!("edges.effects.{}: {err}", style.name())))?,
                None => EffectIn::default(),
            };
            if let Some(color) = own.color.or_else(|| file.anim_color.clone()) {
                e.color = color;
            }
            if let Some(size) = own.size.or(file.anim_size) {
                e.size = size;
            }
            if style.has_direction() {
                e.direction = own.direction.or(file.anim_direction).unwrap_or_default();
            }
            if style.can_shrink() {
                e.shrink = own.shrink.or(file.anim_shrink).unwrap_or_default();
            }
        }
        Ok(Edges {
            on: file.on,
            dwell_ms: file.dwell_ms,
            left: file.left,
            right: file.right,
            top: file.top,
            bottom: file.bottom,
            corner_margin: file.corner_margin,
            place_cursor: file.place_cursor,
            require_control: file.require_control,
            anim_style: file.anim_style,
            effects,
        })
    }
}

impl Edges {
    /// The own settings of an effect; None for None and Random.
    pub fn effect(&self, style: EdgeAnimStyle) -> Option<&EffectSettings> {
        style.effect_index().map(|i| &self.effects[i])
    }

    pub fn effect_mut(&mut self, style: EdgeAnimStyle) -> Option<&mut EffectSettings> {
        style.effect_index().map(|i| &mut self.effects[i])
    }

    /// What the legacy keys show: the chosen effect, or the ring for None and Random.
    fn mirrored(&self) -> &EffectSettings {
        self.effect(self.anim_style).unwrap_or(&self.effects[0])
    }

    /// What the next countdown draws, None for no effect. Random picks one of the
    /// effects, with its own colour and size, and also a direction and shrinking at
    /// random where that effect has them.
    pub fn look(&self) -> Option<EdgeLook> {
        self.look_with(&mut random_below)
    }

    /// `look`, with `pick(n)` giving a number below n for every random choice.
    pub fn look_with(&self, pick: &mut dyn FnMut(usize) -> usize) -> Option<EdgeLook> {
        let mut pick = |n: usize| pick(n) % n;
        let (style, random) = match self.anim_style {
            EdgeAnimStyle::None => return None,
            EdgeAnimStyle::Random => (EDGE_EFFECTS[pick(EDGE_EFFECTS.len())], true),
            style => (style, false),
        };
        let own = self.effect(style)?;
        let direction = if random && style.has_direction() { EDGE_ANIM_DIRECTIONS[pick(3)] } else { own.direction };
        let shrink = if random && style.can_shrink() { pick(2) == 1 } else { own.shrink };
        Some(EdgeLook { style, direction, shrink, color: own.color.clone(), size: own.size })
    }

    /// The effects a legacy `edge.anim_*` setting changes: the chosen one, or every one
    /// when none or Random is chosen.
    fn legacy_targets(&self) -> Vec<EdgeAnimStyle> {
        match self.anim_style.effect_index() {
            Some(_) => vec![self.anim_style],
            None => EDGE_EFFECTS.to_vec(),
        }
    }

    /// Sets one field (`color`, `size`, `direction`, `shrink`) of the given effects.
    fn set_effect_field(&mut self, styles: &[EdgeAnimStyle], key: &str, field: &str, value: &str) -> Result<()> {
        match field {
            "color" => {
                let color = value.trim().to_uppercase();
                styles.iter().for_each(|&s| self.effect_mut(s).into_iter().for_each(|e| e.color = color.clone()));
            }
            "size" => {
                let size = percent(value).with_context(|| format!("{key} must be a percentage, e.g. 125"))?;
                styles.iter().for_each(|&s| self.effect_mut(s).into_iter().for_each(|e| e.size = size));
            }
            "direction" => {
                let direction = EdgeAnimDirection::from_name(value)
                    .with_context(|| format!("{key} must be clockwise, counterclockwise or both"))?;
                let styles: Vec<_> = styles.iter().copied().filter(|s| s.has_direction()).collect();
                anyhow::ensure!(!styles.is_empty(), "{key}: only ring, semicircle and bar have a direction");
                styles.iter().for_each(|&s| self.effect_mut(s).into_iter().for_each(|e| e.direction = direction));
            }
            "shrink" => {
                let shrink: bool = value.trim().parse().with_context(|| format!("{key} must be true or false"))?;
                let styles: Vec<_> = styles.iter().copied().filter(|s| s.can_shrink()).collect();
                anyhow::ensure!(!styles.is_empty(), "{key}: only ring and semicircle can shrink away");
                styles.iter().for_each(|&s| self.effect_mut(s).into_iter().for_each(|e| e.shrink = shrink));
            }
            _ => anyhow::bail!("unknown setting '{key}' (color, size, direction, shrink)"),
        }
        Ok(())
    }

    pub fn channel(&self, edge: Edge) -> u8 {
        match edge {
            Edge::Left => self.left,
            Edge::Right => self.right,
            Edge::Top => self.top,
            Edge::Bottom => self.bottom,
        }
    }

    pub fn channel_mut(&mut self, edge: Edge) -> &mut u8 {
        match edge {
            Edge::Left => &mut self.left,
            Edge::Right => &mut self.right,
            Edge::Top => &mut self.top,
            Edge::Bottom => &mut self.bottom,
        }
    }

    /// The (first) edge that leads to `channel`.
    pub fn edge_for(&self, channel: u8) -> Option<Edge> {
        EDGES.into_iter().find(|&e| channel != 0 && self.channel(e) == channel)
    }

    /// Whether any edge has a channel.
    pub fn any(&self) -> bool {
        EDGES.into_iter().any(|e| self.channel(e) != 0)
    }
}

/// Vibration on a mouse with a haptic motor (MX Master 4): a pulse when the mouse is
/// switched away and when it comes back, and pulses while the cursor waits at a screen
/// edge. Waveforms are the IDs of HID++ feature 0x19B0 (see `device::waveform`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Haptics {
    /// A pulse as the mouse is switched away.
    pub on_switch: bool,
    /// A pulse when the mouse is back. Settings from before 0.9.0 had this under
    /// `on_switch`; `Config::parse` carries it over.
    pub on_arrive: bool,
    /// Pulses while the cursor rests at a screen edge, waiting to switch.
    pub on_edge: bool,
    pub switch_waveform: u8,
    pub arrive_waveform: u8,
    pub edge_waveform: u8,
    /// Strength in percent (1-100), put on the mouse whenever it is here; 0 leaves the
    /// mouse as Logi Options+ set it.
    pub intensity: u8,
    /// The pulses at a screen edge get faster and stronger as the switch nears, instead
    /// of repeating `edge_waveform` evenly.
    pub progressive: bool,
}

impl Default for Haptics {
    fn default() -> Self {
        Haptics {
            on_switch: true,
            on_arrive: true,
            on_edge: true,
            switch_waveform: crate::device::waveform::COMPLETED,
            arrive_waveform: crate::device::waveform::DAMP_COLLISION,
            edge_waveform: crate::device::waveform::SUBTLE_COLLISION,
            intensity: 0,
            progressive: true,
        }
    }
}

/// Names used for the buttons in the settings file and on the command line.
pub fn button_name(cid: u16) -> &'static str {
    match cid {
        BUTTON_WHEEL_MODE => "wheel_mode",
        BUTTON_GESTURE => "gesture",
        BUTTON_MIDDLE => "middle",
        _ => "?",
    }
}

fn button_by_name(name: &str) -> Option<u16> {
    let name = name.trim().to_lowercase();
    BUTTONS
        .into_iter()
        .find(|&c| button_name(c) == name)
        .or_else(|| hex_u16::parse(&name).ok().filter(|c| BUTTONS.contains(c)))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Master switch. When false the buttons are left alone.
    pub enabled: bool,
    /// Which device carries the trigger buttons: "auto" (first mouse that has them),
    /// a wireless product ID such as "B042", or part of the device name.
    pub trigger_device: String,
    /// Devices that move together with the trigger device: ["all"], [] for none, or a
    /// list of wireless product IDs / name fragments, e.g. ["B366", "MX Keys"].
    pub follow: Vec<String>,
    /// Menu and settings language: "ko" or "en". New settings start with the system's.
    pub language: String,
    /// Look for a new version at start and once a day: the Microsoft Store version asks
    /// the Store, the others GitHub.
    pub check_updates: bool,
    /// Install a new version found by that check by itself, then start again.
    pub auto_update: bool,
    /// Per-button settings. (Tables last: TOML writes tables after plain values.)
    pub buttons: Buttons,
    /// Keyboard shortcuts, one per channel.
    pub hotkeys: Hotkeys,
    /// Switching at the screen edge.
    pub edges: Edges,
    /// Vibration on a mouse with a haptic motor.
    pub haptics: Haptics,
    /// Showing where the cursor is.
    pub locate: Locate,
}

/// Keyboard shortcuts that send the group and the master mouse to a channel; pressed on
/// any keyboard connected to this computer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Hotkeys {
    pub on: bool,
    /// Shortcut for channel 1, 2 and 3, e.g. "ctrl+alt+1"; empty for none.
    pub keys: Vec<String>,
}

impl Default for Hotkeys {
    fn default() -> Self {
        Hotkeys { on: false, keys: (1..=3).map(|n| format!("ctrl+alt+{n}")).collect() }
    }
}

impl Hotkeys {
    /// The shortcut of `channel` (1-based); empty when none.
    pub fn key(&self, channel: u8) -> &str {
        self.keys.get(usize::from(channel).wrapping_sub(1)).map_or("", String::as_str)
    }

    pub fn set_key(&mut self, channel: u8, key: &str) {
        self.keys.resize(3, String::new());
        if let Some(k) = self.keys.get_mut(usize::from(channel).wrapping_sub(1)) {
            *k = key.trim().to_lowercase();
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            enabled: true,
            trigger_device: "auto".into(),
            follow: vec!["all".into()],
            language: crate::i18n::system_language().into(),
            check_updates: true,
            auto_update: true,
            buttons: Buttons::default(),
            hotkeys: Hotkeys::default(),
            edges: Edges::default(),
            haptics: Haptics::default(),
            locate: Locate::default(),
        }
    }
}

/// Turns `mode = "fixed"` + `target = n` into `channels = [n]`.
fn fixed_to_channels(t: &mut toml::Table) {
    let fixed = t.get("mode").and_then(|m| m.as_str()) == Some("fixed");
    let target = t.get("target").and_then(|v| v.as_integer()).unwrap_or(1);
    t.remove("mode");
    t.remove("target");
    if fixed {
        t.insert("channels".into(), toml::Value::Array(vec![toml::Value::Integer(target)]));
    }
}

/// Settings of versions before 0.3.1: one mode and channel list for every button.
#[derive(Default, Deserialize)]
#[serde(default)]
struct Legacy {
    mode: Option<String>,
    channels: Option<Vec<u8>>,
    target: Option<u8>,
    #[serde(deserialize_with = "hex_u16_list::deserialize")]
    triggers: Vec<u16>,
    #[serde(deserialize_with = "hex_u16_opt::deserialize")]
    trigger_cid: Option<u16>,
}

impl Config {
    pub fn path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("flockswitch")
            .join("config.toml")
    }

    /// The app was called "hostswitch" before 0.3.0. Moves that settings folder (config,
    /// remembered devices, saved button state) to the new name once, if present.
    pub fn migrate_old_name() {
        let Some(base) = dirs::config_dir() else { return };
        let (old, new) = (base.join("hostswitch"), base.join("flockswitch"));
        if old.is_dir() && !new.exists() {
            let _ = std::fs::rename(&old, &new);
        }
    }

    /// Loads the config, writing the defaults on first run so users have a file to edit.
    pub fn load_or_create() -> Result<Config> {
        let path = Self::path();
        if !path.exists() {
            let cfg = Config::default();
            cfg.save()?;
            return Ok(cfg);
        }
        let text = std::fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
        let cfg = Config::parse(&text).with_context(|| format!("parsing {}", path.display()))?;
        cfg.validate()?;
        Ok(cfg)
    }

    /// Reads a settings file, upgrading the shared mode/channels and `triggers` /
    /// `trigger_cid` of older versions into per-button settings.
    pub fn parse(text: &str) -> Result<Config> {
        let mut table: toml::Table = text.parse()?;
        // Before 0.3.1 a button could "always go to `target`": that is cycling through
        // that one channel. Applies to each button and gesture direction.
        if let Some(toml::Value::Table(buttons)) = table.get_mut("buttons") {
            for (_, button) in buttons.iter_mut() {
                if let toml::Value::Table(b) = button {
                    fixed_to_channels(b);
                    if let Some(toml::Value::Table(moves)) = b.get_mut("moves") {
                        for (_, m) in moves.iter_mut() {
                            if let toml::Value::Table(m) = m {
                                fixed_to_channels(m);
                            }
                        }
                    }
                }
            }
        }
        // Before 0.9.0 one switch covered the pulse on leaving and the one on arriving.
        if let Some(toml::Value::Table(h)) = table.get_mut("haptics") {
            if !h.contains_key("on_arrive") {
                let leaving = h.get("on_switch").and_then(|v| v.as_bool()).unwrap_or(true);
                h.insert("on_arrive".into(), toml::Value::Boolean(leaving));
            }
        }
        let mut cfg: Config = toml::Value::Table(table.clone()).try_into()?;
        if !table.contains_key("buttons") {
            let old: Legacy = toml::Value::Table(table).try_into()?;
            let mut setting = ButtonSetting::default();
            let mut channels = old.channels;
            match old.mode.as_deref() {
                Some("fixed") => channels = Some(vec![old.target.unwrap_or(1)]),
                // "toggle" (between the first two channels) is cycling through those two.
                Some("toggle") => {
                    if let Some(c) = &mut channels {
                        c.truncate(2);
                    }
                }
                _ => {}
            }
            if let Some(c) = channels {
                setting.channels = c;
            }
            let mut on = old.triggers;
            if on.is_empty() {
                on.extend(old.trigger_cid);
            }
            if !on.iter().any(|c| BUTTONS.contains(c)) {
                on = vec![BUTTON_WHEEL_MODE];
            }
            for cid in BUTTONS {
                *cfg.button_mut(cid) = ButtonSetting { on: on.contains(&cid), ..setting.clone() };
            }
        }
        if !["ko", "en"].contains(&cfg.language.as_str()) {
            cfg.language = crate::i18n::system_language().into();
        }
        Ok(cfg)
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::path();
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        std::fs::write(&path, toml::to_string_pretty(self)?)?;
        Ok(())
    }

    /// The setting of a trigger button (`cid` must be one of `BUTTONS`).
    pub fn button(&self, cid: u16) -> &ButtonSetting {
        match cid {
            BUTTON_GESTURE => &self.buttons.gesture,
            BUTTON_MIDDLE => &self.buttons.middle,
            _ => &self.buttons.wheel_mode,
        }
    }

    pub fn button_mut(&mut self, cid: u16) -> &mut ButtonSetting {
        match cid {
            BUTTON_GESTURE => &mut self.buttons.gesture,
            BUTTON_MIDDLE => &mut self.buttons.middle,
            _ => &mut self.buttons.wheel_mode,
        }
    }

    /// Control IDs of the buttons that are switched on.
    pub fn triggers(&self) -> Vec<u16> {
        BUTTONS.into_iter().filter(|&c| self.button(c).on).collect()
    }

    /// The shortcut in slot `n`: channel 1-3, or `hotkey::LOCATE`.
    pub fn shortcut(&self, n: u8) -> &str {
        if n == crate::hotkey::LOCATE { &self.locate.hotkey } else { self.hotkeys.key(n) }
    }

    pub fn set_shortcut(&mut self, n: u8, key: &str) {
        if n == crate::hotkey::LOCATE {
            self.locate.hotkey = key.trim().to_lowercase();
        } else {
            self.hotkeys.set_key(n, key);
        }
    }

    /// Changes one setting from the command line, validating the result.
    /// `channels` applies to every button, or to one with a prefix such as
    /// `gesture.channels`.
    pub fn set(&mut self, key: &str, value: &str) -> Result<()> {
        // Change a copy, so an invalid value leaves the settings as they were.
        let mut next = self.clone();
        next.set_unchecked(key, value)?;
        next.validate()?;
        *self = next;
        Ok(())
    }

    fn set_unchecked(&mut self, key: &str, value: &str) -> Result<()> {
        if key == "hotkeys" || key == "hotkeys.on" {
            self.hotkeys.on = value.parse().context("hotkeys must be true or false")?;
            return Ok(());
        }
        if let Some(what) = key.strip_prefix("haptics.").or((key == "haptics").then_some("on")) {
            let flag = || value.parse::<bool>().with_context(|| format!("{key} must be true or false"));
            let wave = || {
                if value.trim().eq_ignore_ascii_case("random") {
                    return Ok(crate::device::waveform::RANDOM);
                }
                value.trim().parse::<u8>().with_context(|| format!("{key} must be a waveform number (0-31) or random"))
            };
            match what {
                "on" => {
                    self.haptics.on_switch = flag()?;
                    self.haptics.on_arrive = self.haptics.on_switch;
                    self.haptics.on_edge = self.haptics.on_switch;
                }
                "switch" => self.haptics.on_switch = flag()?,
                "arrive" => self.haptics.on_arrive = flag()?,
                "edge" => self.haptics.on_edge = flag()?,
                "switch_waveform" => self.haptics.switch_waveform = wave()?,
                "arrive_waveform" => self.haptics.arrive_waveform = wave()?,
                "edge_waveform" => self.haptics.edge_waveform = wave()?,
                "progressive" => self.haptics.progressive = flag()?,
                "intensity" | "strength" => {
                    let v = value.trim().trim_end_matches('%');
                    self.haptics.intensity = if v.eq_ignore_ascii_case("options+") || v.eq_ignore_ascii_case("default") {
                        0
                    } else {
                        v.parse::<u8>().context("haptics.intensity must be 0 (as Options+ sets it) or 1-100 percent")?
                    };
                }
                _ => anyhow::bail!("unknown setting '{key}' (haptics.switch, haptics.arrive, haptics.edge, haptics.progressive, haptics.intensity, haptics.switch_waveform, haptics.arrive_waveform, haptics.edge_waveform)"),
            }
            return Ok(());
        }
        if let Some(what) = key.strip_prefix("locate.").or((key == "locate").then_some("on")) {
            let flag = || value.parse::<bool>().with_context(|| format!("{key} must be true or false"));
            match what {
                "on" => self.locate.on = flag()?,
                "arrival" => self.locate.on_arrival = flag()?,
                "shake" => self.locate.on_shake = flag()?,
                "shake_time" | "shake_time_ms" => {
                    let ms: u64 = value.trim().trim_end_matches("ms").trim().parse().context("locate.shake_time must be milliseconds, e.g. 1000")?;
                    anyhow::ensure!((200..=5000).contains(&ms), "locate.shake_time must be between 200 and 5000 ms");
                    self.locate.shake_time_ms = ms;
                }
                "shake_key" => {
                    // Strict here, unlike reading a file: only keys Windows has.
                    self.locate.shake_key = match value.trim().to_lowercase().as_str() {
                        "none" => ShakeKey::None,
                        "ctrl" | "control" => ShakeKey::Ctrl,
                        "alt" => ShakeKey::Alt,
                        "shift" => ShakeKey::Shift,
                        _ => anyhow::bail!("locate.shake_key must be none, ctrl, alt or shift"),
                    };
                }
                "shortcut" => self.locate.on_hotkey = flag()?,
                "color" => self.locate.color = value.trim().to_uppercase(),
                "size" => self.locate.size = percent(value).context("locate.size must be a percentage, e.g. 125")?,
                "hotkey" => {
                    let value = if value.eq_ignore_ascii_case("none") { "" } else { value };
                    self.locate.hotkey = value.trim().to_lowercase();
                }
                "style" => {
                    self.locate.style = match value.trim().to_lowercase().as_str() {
                        "ring" => LocateStyle::Ring,
                        "radar" => LocateStyle::Radar,
                        "spotlight" => LocateStyle::Spotlight,
                        "random" => LocateStyle::Random,
                        _ => anyhow::bail!("locate.style must be ring, radar, spotlight or random"),
                    };
                }
                _ => anyhow::bail!("unknown setting '{key}' (locate, locate.style, locate.color, locate.size, locate.hotkey, locate.shortcut, locate.arrival, locate.shake, locate.shake_time, locate.shake_key)"),
            }
            return Ok(());
        }
        if key == "edges" || key == "edges.on" {
            self.edges.on = value.parse().context("edges must be true or false")?;
            return Ok(());
        }
        if let Some(what) = key.strip_prefix("edge.") {
            match what {
                "dwell" | "dwell_ms" => self.edges.dwell_ms = value.trim().parse().context("edge.dwell must be milliseconds, e.g. 500")?,
                "cursor" | "place_cursor" => self.edges.place_cursor = value.parse().context("edge.cursor must be true or false")?,
                "corner" | "corner_margin" => self.edges.corner_margin = value.trim().parse().context("edge.corner must be pixels, e.g. 80")?,
                "control" | "require_control" => self.edges.require_control = value.parse().context("edge.control must be true or false")?,
                "anim" | "anim_style" => {
                    self.edges.anim_style = EdgeAnimStyle::from_name(value)
                        .context("edge.anim must be none, ring, semicircle, bar, pulse, laser, vortex, scan, liquid or random")?;
                }
                // Older names: the chosen effect, or every effect when none or Random is chosen.
                "anim_direction" | "anim_shrink" | "anim_color" | "anim_size" => {
                    let targets = self.edges.legacy_targets();
                    let field = what.trim_start_matches("anim_");
                    self.edges.set_effect_field(&targets, key, field, value)?;
                }
                // One effect's own: edge.ring.color, edge.bar.direction, …
                effect_field if effect_field.contains('.') => {
                    let (name, field) = effect_field.split_once('.').unwrap_or_default();
                    let style = EdgeAnimStyle::from_name(name)
                        .filter(|s| s.effect_index().is_some())
                        .with_context(|| format!("unknown effect '{name}' (ring, semicircle, bar, pulse, laser, vortex, scan, liquid)"))?;
                    self.edges.set_effect_field(&[style], key, field, value)?;
                }
                side => {
                    let edge = Edge::from_name(side).with_context(|| format!("unknown edge '{side}' (left, right, top, bottom)"))?;
                    let channel = if value.eq_ignore_ascii_case("none") || value.eq_ignore_ascii_case("off") {
                        0
                    } else {
                        value.trim().parse::<u8>().context("the edge's channel must be 1, 2, 3 or none")?
                    };
                    *self.edges.channel_mut(edge) = channel;
                }
            }
            return Ok(());
        }
        if let Some(n) = key.strip_prefix("hotkey.") {
            let channel = n.parse::<u8>().ok().filter(|c| (1..=3).contains(c)).context("use hotkey.1, hotkey.2 or hotkey.3")?;
            let value = if value.eq_ignore_ascii_case("none") { "" } else { value };
            self.hotkeys.set_key(channel, value);
            return Ok(());
        }
        let list = |v: &str| -> Vec<String> {
            v.split(',').map(|x| x.trim().to_string()).filter(|x| !x.is_empty()).collect()
        };
        let (which, field) = match key.split_once('.') {
            Some((b, f)) => {
                let cid = button_by_name(b).with_context(|| format!("unknown button '{b}' (wheel_mode, gesture, middle)"))?;
                (vec![cid], f)
            }
            None => (BUTTONS.to_vec(), key),
        };
        match field {
            "enabled" => self.enabled = value.parse().context("enabled must be true or false")?,
            "on" if which.len() == 1 => self.button_mut(which[0]).on = value.parse().context("on must be true or false")?,
            "channels" => {
                let channels: Vec<u8> = list(value)
                    .iter()
                    .map(|c| c.parse::<u8>().context("channels must be numbers like 1,2"))
                    .collect::<Result<_>>()?;
                which.iter().for_each(|&c| self.button_mut(c).channels = channels.clone());
            }
            "follow" => {
                self.follow = if value.eq_ignore_ascii_case("none") { vec![] } else { list(value) }
            }
            "trigger_device" => self.trigger_device = value.trim().to_string(),
            "language" => self.language = value.trim().to_lowercase(),
            "check_updates" => self.check_updates = value.parse().context("check_updates must be true or false")?,
            "auto_update" => self.auto_update = value.parse().context("auto_update must be true or false")?,
            // Which buttons are on, e.g. "wheel_mode,gesture" or "0x00C4,0x00C3".
            // `triggers` / `trigger_cid` are the names older versions used.
            "buttons" | "triggers" | "trigger_cid" => {
                let on: Vec<u16> = list(value)
                    .iter()
                    .map(|v| button_by_name(v).with_context(|| format!("unknown button '{v}' (wheel_mode, gesture, middle)")))
                    .collect::<Result<_>>()?;
                for cid in BUTTONS {
                    self.button_mut(cid).on = on.contains(&cid);
                }
            }
            _ => anyhow::bail!("unknown setting '{key}'"),
        }
        Ok(())
    }

    pub fn validate(&self) -> Result<()> {
        for cid in BUTTONS {
            let b = self.button(cid);
            let name = button_name(cid);
            anyhow::ensure!(b.channels.iter().all(|&c| (1..=3).contains(&c)), "{name}: channels must be between 1 and 3");
            if b.on {
                anyhow::ensure!(!b.channels.is_empty(), "{name}: turn on at least one channel");
            }
            for m in [&b.moves.up, &b.moves.down, &b.moves.left, &b.moves.right] {
                anyhow::ensure!(m.channels.iter().all(|&c| (1..=3).contains(&c)), "{name}: channels must be between 1 and 3");
                if b.on && m.on {
                    anyhow::ensure!(!m.channels.is_empty(), "{name}: turn on at least one channel");
                }
            }
        }
        anyhow::ensure!(!self.triggers().is_empty(), "turn on at least one button");
        anyhow::ensure!(["ko", "en"].contains(&self.language.as_str()), "language must be ko or en");
        for e in EDGES {
            anyhow::ensure!(self.edges.channel(e) <= 3, "edge.{}: the channel must be 1, 2, 3 or none", e.name());
        }
        anyhow::ensure!((100..=5000).contains(&self.edges.dwell_ms), "edge.dwell must be between 100 and 5000 ms");
        anyhow::ensure!(self.edges.corner_margin <= 1000, "edge.corner must be at most 1000 pixels");
        for (style, e) in EDGE_EFFECTS.into_iter().zip(&self.edges.effects) {
            let name = style.name();
            anyhow::ensure!(parse_color(&e.color).is_some(), "edge.{name}.color must be a colour like #FFCC00");
            anyhow::ensure!(EFFECT_SIZES.contains(&e.size) || (50..=200).contains(&e.size), "edge.{name}.size must be between 50 and 200 percent");
        }
        anyhow::ensure!(parse_color(&self.locate.color).is_some(), "locate.color must be a colour like #FFCC00");
        // Lenient: a file with a time outside what the window offers still loads.
        anyhow::ensure!((100..=10_000).contains(&self.locate.shake_time_ms), "locate.shake_time must be between 200 and 5000 ms");
        anyhow::ensure!(EFFECT_SIZES.contains(&self.locate.size) || (50..=200).contains(&self.locate.size), "locate.size must be between 50 and 200 percent");
        for w in [self.haptics.switch_waveform, self.haptics.arrive_waveform, self.haptics.edge_waveform] {
            anyhow::ensure!(w <= 31 || w == crate::device::waveform::RANDOM, "haptics: waveforms are numbered 0 to 31, or random");
        }
        anyhow::ensure!(self.haptics.intensity <= 100, "haptics.intensity must be 0 (as Options+ sets it) or 1-100 percent");
        anyhow::ensure!(self.hotkeys.keys.len() <= 3, "hotkeys: one shortcut per channel (1-3)");
        let mut seen = Vec::new();
        for key in self.hotkeys.keys.iter().chain([&self.locate.hotkey]).filter(|k| !k.is_empty()) {
            let parsed = crate::hotkey::parse(key)?;
            anyhow::ensure!(!seen.contains(&parsed), "shortcut '{key}' is used for two channels");
            seen.push(parsed);
        }
        Ok(())
    }

    /// True when every device follows ("all").
    pub fn follows_all(&self) -> bool {
        self.follow.iter().any(|f| f.eq_ignore_ascii_case("all"))
    }

    /// Ticks or unticks one device in the group. "all" is expanded to the given device
    /// list first, and collapsed back to "all" when every device is ticked again.
    pub fn toggle_follow(&mut self, devices: &[String], key: &str) {
        let mut set: Vec<String> = if self.follows_all() { devices.to_vec() } else { self.follow.clone() };
        if let Some(i) = set.iter().position(|f| f.eq_ignore_ascii_case(key)) {
            set.remove(i);
        } else {
            set.push(key.to_string());
        }
        let every = !devices.is_empty() && devices.iter().all(|d| set.iter().any(|f| f.eq_ignore_ascii_case(d)));
        self.follow = if every { vec!["all".into()] } else { set };
    }

    /// Whether a device with this product ID / name should follow the trigger device.
    pub fn follows(&self, wpid: Option<u16>, name: &str) -> bool {
        self.follow.iter().any(|f| {
            let f = f.trim();
            f.eq_ignore_ascii_case("all")
                || wpid.is_some_and(|w| f.eq_ignore_ascii_case(&format!("{w:04X}")))
                || (!f.is_empty() && name.to_lowercase().contains(&f.to_lowercase()))
        })
    }
}

/// Control IDs are written as "0x00C4"; either that form or a plain number is accepted.
mod hex_u16 {
    use serde::Deserialize;

    #[derive(Deserialize)]
    #[serde(untagged)]
    pub enum Raw {
        Num(u16),
        Text(String),
    }

    pub fn parse(t: &str) -> Result<u16, std::num::ParseIntError> {
        let t = t.trim();
        let hex = t.strip_prefix("0x").or_else(|| t.strip_prefix("0X")).unwrap_or(t);
        u16::from_str_radix(hex, 16)
    }

    pub fn from_raw<E: serde::de::Error>(r: Raw) -> Result<u16, E> {
        match r {
            Raw::Num(n) => Ok(n),
            Raw::Text(t) => parse(&t).map_err(E::custom),
        }
    }
}

mod hex_u16_list {
    use super::hex_u16::{self, Raw};
    use serde::{Deserialize, Deserializer};

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<u16>, D::Error> {
        Vec::<Raw>::deserialize(d)?.into_iter().map(hex_u16::from_raw).collect()
    }
}

mod hex_u16_opt {
    use super::hex_u16::{self, Raw};
    use serde::{Deserialize, Deserializer};

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Option<u16>, D::Error> {
        Option::<Raw>::deserialize(d)?.map(hex_u16::from_raw).transpose()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn button(channels: &[u8]) -> ButtonSetting {
        ButtonSetting { on: true, channels: channels.to_vec(), ..ButtonSetting::default() }
    }

    fn press(b: &ButtonSetting, current0: u8, count: u8) -> Option<u8> {
        b.channel_after(Gesture::Press, current0, count)
    }

    #[test]
    fn cycle_wraps_like_the_hardware_button() {
        let b = button(&[1, 2, 3]);
        assert_eq!(press(&b, 0, 3), Some(2));
        assert_eq!(press(&b, 1, 3), Some(3));
        assert_eq!(press(&b, 2, 3), Some(1));
    }

    #[test]
    fn cycle_skips_channels_the_device_lacks() {
        assert_eq!(press(&button(&[1, 2, 3]), 1, 2), Some(1));
    }

    #[test]
    fn cycle_from_unlisted_channel_goes_to_first() {
        assert_eq!(press(&button(&[1, 2]), 2, 3), Some(1));
    }

    #[test]
    fn two_channels_bounce() {
        let b = button(&[1, 2]);
        assert_eq!(press(&b, 0, 3), Some(2));
        assert_eq!(press(&b, 1, 3), Some(1));
        assert_eq!(press(&b, 2, 3), Some(1));
    }

    #[test]
    fn one_channel_always_goes_there() {
        let b = button(&[2]);
        assert_eq!(press(&b, 1, 3), None);
        assert_eq!(press(&b, 0, 3), Some(2));
        assert_eq!(press(&b, 2, 3), Some(2));
        assert_eq!(press(&button(&[3]), 0, 2), None); // the device has no channel 3
    }

    #[test]
    fn gestures() {
        assert_eq!(Gesture::from_movement(10, -20), Gesture::Press);
        assert_eq!(Gesture::from_movement(300, 40), Gesture::Right);
        assert_eq!(Gesture::from_movement(-300, 100), Gesture::Left);
        assert_eq!(Gesture::from_movement(20, -200), Gesture::Up);
        assert_eq!(Gesture::from_movement(-30, 200), Gesture::Down);
        let mut b = button(&[1, 2]);
        assert!(!b.uses_moves());
        b.moves.left = MoveSetting { on: true, channels: vec![1] };
        b.moves.right = MoveSetting { on: true, channels: vec![3] };
        b.moves.up = MoveSetting { on: true, channels: vec![1, 3] };
        b.moves.down.channels = vec![2]; // set but off
        assert_eq!(b.channel_after(Gesture::Press, 0, 3), Some(2));
        assert_eq!(b.channel_after(Gesture::Right, 0, 3), Some(3));
        assert_eq!(b.channel_after(Gesture::Left, 0, 3), None); // already on 1
        assert_eq!(b.channel_after(Gesture::Up, 0, 3), Some(3));
        assert_eq!(b.channel_after(Gesture::Up, 2, 3), Some(1));
        assert_eq!(b.channel_after(Gesture::Down, 0, 3), None); // off
        assert!(b.uses_moves());
    }

    #[test]
    fn moves_round_trip_only_when_set() {
        let mut c = Config::default();
        assert!(!toml::to_string_pretty(&c).unwrap().contains("moves"));
        c.buttons.gesture.moves.left = MoveSetting { on: true, channels: vec![2] };
        let text = toml::to_string_pretty(&c).unwrap();
        assert!(text.contains("[buttons.gesture.moves.left]"), "{text}");
        assert_eq!(Config::parse(&text).unwrap().buttons, c.buttons);
    }

    #[test]
    fn buttons_round_trip() {
        let mut c = Config::default();
        c.buttons.gesture = ButtonSetting { on: true, channels: vec![3], ..ButtonSetting::default() };
        let text = toml::to_string_pretty(&c).unwrap();
        assert!(text.contains("[buttons.gesture]"), "{text}");
        assert!(!text.contains("mode =") && !text.contains("target"), "{text}");
        let back = Config::parse(&text).unwrap();
        assert_eq!(back.buttons, c.buttons);
        assert_eq!(back.triggers(), vec![BUTTON_WHEEL_MODE, BUTTON_GESTURE]);
    }

    #[test]
    fn fixed_target_becomes_one_channel() {
        let text = "[buttons.gesture]\non = true\nmode = \"fixed\"\ntarget = 3\nchannels = [1, 2]\n\
                    [buttons.gesture.moves.left]\non = true\nmode = \"fixed\"\ntarget = 2\n\
                    [buttons.wheel_mode]\non = true\nmode = \"cycle\"\ntarget = 3\nchannels = [1, 2]\n";
        let c = Config::parse(text).unwrap();
        assert_eq!(c.buttons.gesture.channels, vec![3]);
        assert_eq!(c.buttons.gesture.moves.left.channels, vec![2]);
        assert_eq!(c.buttons.wheel_mode.channels, vec![1, 2]);
    }

    #[test]
    fn shared_settings_of_older_versions_become_per_button() {
        let c = Config::parse("mode = \"fixed\"\ntarget = 2\nchannels = [1, 2]\ntriggers = [\"0x00C4\", \"0x0052\"]").unwrap();
        assert_eq!(c.triggers(), vec![BUTTON_WHEEL_MODE, BUTTON_MIDDLE]);
        for cid in BUTTONS {
            assert_eq!(c.button(cid).channels, vec![2]);
        }
        // "toggle" (first two channels) became cycling through those two.
        let c = Config::parse("mode = \"toggle\"\nchannels = [1, 3, 2]").unwrap();
        assert_eq!(c.buttons.wheel_mode.channels, vec![1, 3]);
        // A single trigger_cid (0.3.0), or nothing at all.
        assert_eq!(Config::parse("trigger_cid = \"0x00C3\"").unwrap().triggers(), vec![BUTTON_GESTURE]);
        assert_eq!(Config::parse("trigger_cid = 196").unwrap().triggers(), vec![BUTTON_WHEEL_MODE]);
        assert_eq!(Config::parse("enabled = true").unwrap().triggers(), vec![BUTTON_WHEEL_MODE]);
        // "auto" was offered for a while; it becomes the system's language.
        let lang = Config::parse("language = \"auto\"").unwrap().language;
        assert!(lang == "ko" || lang == "en");
    }

    #[test]
    fn set_from_command_line() {
        let mut c = Config::default();
        c.set("channels", "1, 2").unwrap();
        assert!(BUTTONS.iter().all(|&b| c.button(b).channels == vec![1, 2]));
        c.set("gesture.channels", "3").unwrap();
        assert_eq!(c.buttons.gesture.channels, vec![3]);
        assert_eq!(c.buttons.wheel_mode.channels, vec![1, 2]);
        c.set("middle.on", "true").unwrap();
        assert_eq!(c.triggers(), vec![BUTTON_WHEEL_MODE, BUTTON_MIDDLE]);
        c.set("buttons", "gesture, 0x00c4").unwrap();
        assert_eq!(c.triggers(), vec![BUTTON_WHEEL_MODE, BUTTON_GESTURE]);
        assert!(c.set("buttons", "").is_err());
        assert!(c.set("mode", "fixed").is_err());
        assert!(c.set("channels", "").is_err()); // a button that is on needs a channel
        c.set("follow", "none").unwrap();
        assert!(c.follow.is_empty());
        assert!(c.set("channels", "4").is_err());
        assert!(c.set("thumb.channels", "1").is_err());
        assert!(c.set("bogus", "1").is_err());
    }

    #[test]
    fn untick_one_device_from_all() {
        let mut c = Config::default(); // follow = ["all"]
        let devices = vec!["B366".to_string(), "B377".to_string()];
        c.toggle_follow(&devices, "B366");
        assert_eq!(c.follow, vec!["B377".to_string()]);
    }

    #[test]
    fn ticking_the_last_one_collapses_to_all() {
        let mut c = Config { follow: vec!["B377".into()], ..Config::default() };
        let devices = vec!["B366".to_string(), "B377".to_string()];
        c.toggle_follow(&devices, "B366");
        assert_eq!(c.follow, vec!["all".to_string()]);
    }

    #[test]
    fn follow_matching() {
        let mut c = Config::default();
        assert!(c.follows(Some(0xB366), "MX Mechanical"));
        c.follow = vec!["b366".into()];
        assert!(c.follows(Some(0xB366), "x"));
        assert!(!c.follows(Some(0xB042), "MX Master 4"));
        c.follow = vec!["keys".into()];
        assert!(c.follows(None, "MX Keys S"));
        c.follow = vec![];
        assert!(!c.follows(Some(0xB366), "MX Mechanical"));
    }

    #[test]
    fn edges() {
        let mut c = Config::default();
        assert!(!c.edges.on && !c.edges.any());
        c.set("edges", "true").unwrap();
        c.set("edge.right", "2").unwrap();
        c.set("edge.left", "none").unwrap();
        c.set("edge.dwell", "800").unwrap();
        assert!(c.edges.on && c.edges.any());
        assert_eq!(c.edges.channel(Edge::Right), 2);
        assert_eq!(c.edges.edge_for(2), Some(Edge::Right));
        assert_eq!(c.edges.edge_for(1), None);
        assert_eq!(c.edges.dwell_ms, 800);
        assert!(c.set("edge.right", "4").is_err());
        assert!(c.set("edge.dwell", "50").is_err());
        assert!(c.set("edge.middle", "1").is_err());
        let back = Config::parse(&toml::to_string_pretty(&c).unwrap()).unwrap();
        assert_eq!(back.edges, c.edges);
        // Settings from before edges existed have them off.
        assert_eq!(Config::parse("enabled = true").unwrap().edges, Edges::default());
    }

    #[test]
    fn haptics() {
        let mut c = Config::default();
        assert!(c.haptics.on_switch && c.haptics.on_edge);
        c.set("haptics.edge", "false").unwrap();
        c.set("haptics.switch_waveform", "12").unwrap();
        assert!(c.haptics.on_switch && !c.haptics.on_edge);
        assert_eq!(c.haptics.switch_waveform, 12);
        assert!(c.set("haptics.edge_waveform", "40").is_err());
        c.set("haptics.intensity", "75%").unwrap();
        assert_eq!(c.haptics.intensity, 75);
        c.set("haptics.intensity", "options+").unwrap();
        assert_eq!(c.haptics.intensity, 0);
        assert!(c.set("haptics.intensity", "120").is_err());
        assert!(c.set("haptics.bogus", "1").is_err());
        assert!(c.haptics.progressive);
        c.set("haptics.progressive", "false").unwrap();
        assert!(!c.haptics.progressive);
        c.set("haptics.arrive_waveform", "random").unwrap();
        assert_eq!(c.haptics.arrive_waveform, crate::device::waveform::RANDOM);
        c.set("haptics.arrive", "false").unwrap();
        assert!(c.haptics.on_switch && !c.haptics.on_arrive);
        c.set("haptics", "false").unwrap();
        assert!(!c.haptics.on_switch && !c.haptics.on_arrive && !c.haptics.on_edge);
        // Older settings: the arrival pulse followed the switch pulse.
        let old = Config::parse("[haptics]\non_switch = false\non_edge = true\n").unwrap();
        assert!(!old.haptics.on_arrive && old.haptics.on_edge);
        assert!(Config::parse("[haptics]\non_edge = true\n").unwrap().haptics.on_arrive);
        let back = Config::parse(&toml::to_string_pretty(&c).unwrap()).unwrap();
        assert_eq!(back.haptics, c.haptics);
        assert_eq!(Config::parse("enabled = true").unwrap().haptics, Haptics::default());
    }

    #[test]
    fn edge_countdown_settings() {
        let mut c = Config::default();
        assert!(!c.edges.require_control);
        assert_eq!(c.edges.anim_style, EdgeAnimStyle::Ring);
        c.set("edge.control", "true").unwrap();
        c.set("edge.anim", "Vortex").unwrap();
        // The vortex has neither a direction nor shrinking.
        assert!(c.set("edge.anim_direction", "both").is_err());
        assert!(c.set("edge.anim_shrink", "true").is_err());
        c.set("edge.anim", "ring").unwrap();
        c.set("edge.anim_direction", "both").unwrap();
        c.set("edge.anim_shrink", "true").unwrap();
        let ring = c.edges.effect(EdgeAnimStyle::Ring).unwrap().clone();
        assert!(c.edges.require_control && ring.shrink);
        assert_eq!(ring.direction, EdgeAnimDirection::Both);
        assert!(c.set("edge.anim", "sparkle").is_err());
        assert_eq!(ring.color, DEFAULT_EFFECT_COLOR);
        c.set("edge.anim_color", "#0a84ff").unwrap();
        let color = &c.edges.effect(EdgeAnimStyle::Ring).unwrap().color;
        assert_eq!(color, "#0A84FF");
        assert_eq!(parse_color(color), Some([0x0A, 0x84, 0xFF]));
        assert_eq!(format_color([0x0A, 0x84, 0xFF]), "#0A84FF");
        for bad in ["red", "#12345", "#GGGGGG", "0A84FF", "#한글한"] {
            assert!(c.set("edge.anim_color", bad).is_err(), "{bad}");
            assert!(c.set("edge.bar.color", bad).is_err(), "{bad}");
        }
        assert!(c.set("edge.anim_direction", "up").is_err());
        c.set("edge.anim", "vortex").unwrap();
        let back = Config::parse(&toml::to_string_pretty(&c).unwrap()).unwrap();
        assert_eq!(back.edges, c.edges);
        c.set("edge.anim", "random").unwrap();
        for _ in 0..200 {
            let picked = c.edges.anim_style.resolved();
            assert!(picked != EdgeAnimStyle::None && picked != EdgeAnimStyle::Random, "{picked:?}");
        }
        assert_eq!(EdgeAnimStyle::Bar.resolved(), EdgeAnimStyle::Bar);
        // The table of drawing functions in overlay_render.rs is indexed by these numbers.
        for (i, style) in EDGE_ANIM_STYLES.into_iter().enumerate() {
            assert_eq!(style as usize, i);
            assert_eq!(EdgeAnimStyle::from_name(style.name()), Some(style));
        }
    }

    #[test]
    fn settings_from_before_each_effect_had_its_own() {
        // 1.7: one colour, size, direction and shrinking for every effect.
        let old = "[edges]\nanim_style = \"bar\"\nanim_direction = \"both\"\nanim_shrink = true\nanim_color = \"#0A84FF\"\nanim_size = 125\n";
        let c = Config::parse(old).unwrap();
        c.validate().unwrap();
        for (style, e) in EDGE_EFFECTS.into_iter().zip(&c.edges.effects) {
            assert_eq!((e.color.as_str(), e.size), ("#0A84FF", 125), "{style:?}");
            let direction = if style.has_direction() { EdgeAnimDirection::Both } else { EdgeAnimDirection::Clockwise };
            assert_eq!(e.direction, direction, "{style:?}");
            assert_eq!(e.shrink, style.can_shrink(), "{style:?}");
        }
        // A table with some fields: the rest still come from the older keys.
        let mixed = format!("{old}[edges.effects.ring]\ncolor = \"#FF3B30\"\n");
        let c = Config::parse(&mixed).unwrap();
        let ring = c.edges.effect(EdgeAnimStyle::Ring).unwrap();
        assert_eq!((ring.color.as_str(), ring.size, ring.direction, ring.shrink), ("#FF3B30", 125, EdgeAnimDirection::Both, true));
        assert_eq!(c.edges.effect(EdgeAnimStyle::Bar).unwrap().color, "#0A84FF");
        // Nothing at all: the defaults.
        assert_eq!(Config::parse("enabled = true").unwrap().edges.effects, <[EffectSettings; 8]>::default());
    }

    #[test]
    fn each_effect_keeps_its_own_settings() {
        let mut c = Config::default();
        c.set("edge.ring.direction", "both").unwrap();
        c.set("edge.ring.color", "#ff3b30").unwrap();
        c.set("edge.bar.size", "150%").unwrap();
        c.set("edge.semicircle.shrink", "true").unwrap();
        let get = |c: &Config, s| c.edges.effect(s).unwrap().clone();
        assert_eq!(get(&c, EdgeAnimStyle::Ring).direction, EdgeAnimDirection::Both);
        assert_eq!(get(&c, EdgeAnimStyle::Bar).direction, EdgeAnimDirection::Clockwise, "the bar is not the ring");
        assert_eq!(get(&c, EdgeAnimStyle::Bar).color, DEFAULT_EFFECT_COLOR);
        assert_eq!(get(&c, EdgeAnimStyle::Bar).size, 150);
        assert_eq!(get(&c, EdgeAnimStyle::Ring).size, 100);
        assert!(get(&c, EdgeAnimStyle::Semicircle).shrink && !get(&c, EdgeAnimStyle::Ring).shrink);
        for bad in ["edge.pulse.direction", "edge.laser.shrink", "edge.bar.shrink"] {
            assert!(c.set(bad, "true").is_err(), "{bad}");
        }
        assert!(c.set("edge.ring.size", "300").is_err());
        assert!(c.set("edge.ring.speed", "1").is_err());
        assert!(c.set("edge.sparkle.color", "#FFFFFF").is_err());
        assert!(c.set("edge.random.color", "#FFFFFF").is_err());
        // The older names change the chosen effect only…
        c.set("edge.anim", "bar").unwrap();
        c.set("edge.anim_color", "#34C759").unwrap();
        assert_eq!(get(&c, EdgeAnimStyle::Bar).color, "#34C759");
        assert_eq!(get(&c, EdgeAnimStyle::Ring).color, "#FF3B30");
        // …or every effect when none or Random is chosen.
        c.set("edge.anim", "random").unwrap();
        c.set("edge.anim_size", "75").unwrap();
        assert!(c.edges.effects.iter().all(|e| e.size == 75));
        c.set("edge.anim_direction", "counterclockwise").unwrap();
        for style in EDGE_EFFECTS {
            let want = if style.has_direction() { EdgeAnimDirection::CounterClockwise } else { EdgeAnimDirection::Clockwise };
            assert_eq!(get(&c, style).direction, want, "{style:?}");
        }
    }

    #[test]
    fn effects_round_trip_with_the_older_keys_mirroring_the_chosen_one() {
        let mut c = Config::default();
        c.set("edge.anim", "semicircle").unwrap();
        c.set("edge.semicircle.color", "#FF2D92").unwrap();
        c.set("edge.semicircle.size", "125").unwrap();
        c.set("edge.semicircle.direction", "counterclockwise").unwrap();
        c.set("edge.semicircle.shrink", "true").unwrap();
        c.set("edge.liquid.color", "#FFFFFF").unwrap();
        let text = toml::to_string_pretty(&c).unwrap();
        for line in ["anim_color = \"#FF2D92\"", "anim_size = 125", "anim_direction = \"counterclockwise\"", "anim_shrink = true"] {
            assert!(text.contains(line), "{line} in {text}");
        }
        for name in EDGE_EFFECTS.map(EdgeAnimStyle::name) {
            assert!(text.contains(&format!("[edges.effects.{name}]")), "{name} in {text}");
        }
        // Direction only where there is one, shrinking only for the ring and semicircle.
        let table: toml::Table = text.parse().unwrap();
        let effects = table["edges"]["effects"].as_table().unwrap();
        for style in EDGE_EFFECTS {
            let e = effects[style.name()].as_table().unwrap();
            assert_eq!(e.contains_key("direction"), style.has_direction(), "{style:?}");
            assert_eq!(e.contains_key("shrink"), style.can_shrink(), "{style:?}");
            assert!(e.contains_key("color") && e.contains_key("size"));
        }
        let back = Config::parse(&text).unwrap();
        assert_eq!(back.edges, c.edges);
        // None and Random mirror the ring.
        c.set("edge.anim", "random").unwrap();
        let text = toml::to_string_pretty(&c).unwrap();
        assert!(text.contains("anim_color = \"#FFCC00\""), "{text}");
        assert_eq!(Config::parse(&text).unwrap().edges, c.edges);
    }

    #[test]
    fn unknown_effects_and_fields_are_passed_over() {
        let text = "[edges]\nanim_style = \"ring\"\n\
                    [edges.effects.ring]\ncolor = \"#0A84FF\"\nsparkles = 12\nspeed = \"fast\"\n\
                    [edges.effects.sparkle]\ncolor = 5\nsize = \"huge\"\n\
                    [edges.effects.pulse]\ndirection = \"both\"\nshrink = true\nsize = 75\n";
        let c = Config::parse(text).unwrap();
        c.validate().unwrap();
        assert_eq!(c.edges.effect(EdgeAnimStyle::Ring).unwrap().color, "#0A84FF");
        // The pulse has no direction or shrinking: those are left at their defaults.
        let pulse = c.edges.effect(EdgeAnimStyle::Pulse).unwrap();
        assert_eq!((pulse.size, pulse.direction, pulse.shrink), (75, EdgeAnimDirection::Clockwise, false));
        // A known effect with a value of the wrong kind is refused, as elsewhere.
        assert!(Config::parse("[edges.effects.bar]\nsize = \"big\"\n").is_err());
        let bad = Config::parse("[edges.effects.bar]\ncolor = \"blue\"\n").unwrap();
        assert!(bad.validate().is_err());
    }

    #[test]
    fn random_varies_the_options_and_keeps_colour_and_size() {
        let mut c = Config::default();
        c.set("edge.ring.color", "#FF3B30").unwrap();
        c.set("edge.ring.size", "150").unwrap();
        c.set("edge.ring.direction", "both").unwrap();
        c.set("edge.vortex.color", "#34C759").unwrap();
        // A chosen effect: exactly its saved settings, whatever the dice say.
        c.set("edge.anim", "ring").unwrap();
        let look = c.edges.look_with(&mut |n| n - 1).unwrap();
        assert_eq!(look, EdgeLook { style: EdgeAnimStyle::Ring, direction: EdgeAnimDirection::Both, shrink: false, color: "#FF3B30".into(), size: 150 });
        // Random: the effect, its direction and its shrinking come from the dice.
        c.set("edge.anim", "random").unwrap();
        let mut rolls = vec![0, 1, 1].into_iter(); // ring, counter-clockwise, shrinking
        let look = c.edges.look_with(&mut |_| rolls.next().unwrap()).unwrap();
        assert_eq!(look, EdgeLook { style: EdgeAnimStyle::Ring, direction: EdgeAnimDirection::CounterClockwise, shrink: true, color: "#FF3B30".into(), size: 150 });
        // An effect without options rolls only once; colour and size are its own.
        let mut rolls = vec![5].into_iter();
        let look = c.edges.look_with(&mut |_| rolls.next().unwrap()).unwrap();
        assert_eq!(look.style, EdgeAnimStyle::Vortex);
        assert_eq!((look.color.as_str(), look.size, look.shrink), ("#34C759", 100, false));
        // The bar rolls a direction but never shrinks.
        let mut rolls = vec![2, 2].into_iter();
        let look = c.edges.look_with(&mut |_| rolls.next().unwrap()).unwrap();
        assert_eq!((look.style, look.direction, look.shrink), (EdgeAnimStyle::Bar, EdgeAnimDirection::Both, false));
        // Over many real draws every effect, direction and shrinking turns up.
        let looks: Vec<EdgeLook> = (0..2000).map(|_| c.edges.look().unwrap()).collect();
        for style in EDGE_EFFECTS {
            assert!(looks.iter().any(|l| l.style == style), "{style:?}");
        }
        for d in EDGE_ANIM_DIRECTIONS {
            assert!(looks.iter().any(|l| l.style == EdgeAnimStyle::Semicircle && l.direction == d), "{d:?}");
        }
        assert!(looks.iter().any(|l| l.style == EdgeAnimStyle::Ring && l.shrink));
        assert!(looks.iter().any(|l| l.style == EdgeAnimStyle::Ring && !l.shrink));
        assert!(looks.iter().all(|l| l.style.can_shrink() || !l.shrink));
        c.set("edge.anim", "none").unwrap();
        assert_eq!(c.edges.look(), None);
    }

    #[test]
    fn every_effect_draws_in_its_own_colour_and_size() {
        use crate::overlay_render::{draw_edge, EdgeDraw, EDGE_TOP, STRIP_DEPTH};
        use tiny_skia::Pixmap;
        let blue = [0x0A, 0x84, 0xFF];
        let draw = |style: EdgeAnimStyle, direction: EdgeAnimDirection, shrink: bool, size: f64| {
            let mut pixmap = Pixmap::new(1920, STRIP_DEPTH as u32).unwrap();
            let d = EdgeDraw {
                edge: EDGE_TOP,
                style: style as i32,
                direction: direction as i32,
                shrink,
                cursor: (900.0, 0.0),
                scale: 1.0,
                t: 1.0,
                duration: 2.0,
                color: blue,
                size,
            };
            draw_edge(&mut pixmap, &d);
            pixmap
        };
        let painted = |p: &Pixmap| p.pixels().iter().filter(|px| px.alpha() > 0).count();
        for style in EDGE_EFFECTS {
            let normal = draw(style, EdgeAnimDirection::Clockwise, false, 1.0);
            // Only nearly opaque pixels, and some room: where many faint layers overlap
            // (premultiplied, 8 bits), the rounding of each adds up.
            let solid: Vec<_> = normal.pixels().iter().filter(|px| px.alpha() >= 230).collect();
            assert!(solid.len() > 5, "{style:?} paints solidly");
            for px in solid {
                let c = px.demultiply();
                let near = |a: u8, b: u8| a.abs_diff(b) <= 12;
                assert!(near(c.red(), blue[0]) && near(c.green(), blue[1]) && near(c.blue(), blue[2]), "{style:?}: {c:?}");
            }
            let large = draw(style, EdgeAnimDirection::Clockwise, false, 1.5);
            let ratio = painted(&large) as f64 / painted(&normal) as f64;
            assert!(ratio > 1.2, "{style:?} grows with its size: {ratio}");
            if style.has_direction() {
                let pixels = |d| draw(style, d, false, 1.0).data().to_vec();
                let (cw, ccw, both) = (pixels(EdgeAnimDirection::Clockwise), pixels(EdgeAnimDirection::CounterClockwise), pixels(EdgeAnimDirection::Both));
                assert!(cw != ccw && cw != both && ccw != both, "{style:?} follows its direction");
            }
            if style.can_shrink() {
                let shrunk = draw(style, EdgeAnimDirection::Clockwise, true, 1.0);
                assert_ne!(shrunk.data(), normal.data(), "{style:?} shrinks");
            }
        }
    }

    #[test]
    fn locate() {
        let mut c = Config::default();
        assert!(!c.locate.on);
        assert_eq!(c.shortcut(crate::hotkey::LOCATE), "ctrl+alt+l");
        c.set("locate", "true").unwrap();
        c.set("locate.style", "spotlight").unwrap();
        c.set("locate.shake", "true").unwrap();
        c.set("locate.hotkey", "ctrl+alt+shift+k").unwrap();
        assert!(c.locate.on && c.locate.on_shake);
        assert_eq!(c.locate.style, LocateStyle::Spotlight);
        assert_eq!(c.locate.hotkey, "ctrl+alt+shift+k");
        assert!(c.set("locate.hotkey", "ctrl+alt+1").is_err(), "already channel 1's");
        assert!(c.set("locate.style", "laser").is_err());
        c.set("locate.style", "random").unwrap();
        assert!((0..100).all(|_| c.locate.style.resolved() != LocateStyle::Random));
        c.set("locate.style", "spotlight").unwrap();
        c.set("locate.size", "150%").unwrap();
        c.set("edge.anim_size", "75").unwrap();
        assert_eq!((c.locate.size, c.edges.effect(EdgeAnimStyle::Ring).unwrap().size), (150, 75));
        assert!(c.set("locate.size", "300").is_err());
        c.set("locate.color", "#34c759").unwrap();
        assert_eq!(c.locate.color, "#34C759");
        assert!(c.set("locate.color", "green").is_err());
        assert!(c.set("locate.bogus", "1").is_err());
        c.set("locate.hotkey", "none").unwrap();
        assert_eq!(c.locate.hotkey, "");
        assert_eq!((c.locate.shake_time_ms, c.locate.shake_key), (1000, ShakeKey::None));
        c.set("locate.shake_time", "1500").unwrap();
        c.set("locate.shake_key", "Shift").unwrap();
        assert_eq!((c.locate.shake_time_ms, c.locate.shake_key), (1500, ShakeKey::Shift));
        assert!(c.set("locate.shake_time", "50").is_err());
        assert!(c.set("locate.shake_time", "fast").is_err());
        assert!(c.set("locate.shake_key", "win").is_err());
        assert!(toml::to_string_pretty(&c).unwrap().contains("shake_key = \"shift\""));
        let back = Config::parse(&toml::to_string_pretty(&c).unwrap()).unwrap();
        assert_eq!(back.locate, c.locate);
        assert_eq!(Config::parse("enabled = true").unwrap().locate, Locate::default());
    }

    #[test]
    fn hotkeys() {
        let mut c = Config::default();
        assert!(!c.hotkeys.on);
        assert_eq!(c.hotkeys.key(2), "ctrl+alt+2");
        c.set("hotkeys", "true").unwrap();
        c.set("hotkey.3", "Ctrl+Shift+F5").unwrap();
        assert_eq!(c.hotkeys.key(3), "ctrl+shift+f5");
        c.set("hotkey.1", "none").unwrap();
        assert_eq!(c.hotkeys.key(1), "");
        assert!(c.set("hotkey.3", "ctrl+alt+2").is_err(), "same shortcut twice");
        assert!(c.set("hotkey.2", "ctrl+alt+").is_err());
        assert!(c.set("hotkey.4", "ctrl+alt+4").is_err());
        let back = Config::parse(&toml::to_string_pretty(&c).unwrap()).unwrap();
        assert_eq!(back.hotkeys, c.hotkeys);
        // Settings from before shortcuts existed get them off.
        assert!(!Config::parse("enabled = true").unwrap().hotkeys.on);
    }
}
