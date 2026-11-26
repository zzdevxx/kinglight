use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EncoderMode {
    #[default]
    Normal,
    Fine,
    Ultra,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Interleave {
    None,
    Even,
    Odd,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ShapeDirection {
    Left,
    Right,
    Bounce,
    Cross,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ShapeParams {
    pub speed: f64,
    pub bpm: Option<f64>,
    pub size: f64,
    pub spread: f64,
    pub repeat: u32,
    pub interleave: Interleave,
    pub direction: ShapeDirection,
    pub amplitude: f64,
}

impl Default for ShapeParams {
    fn default() -> Self {
        Self {
            speed: 1.0,
            bpm: Some(6.0),
            size: 25.0 / 255.0,
            spread: 1.0,
            repeat: 8,
            interleave: Interleave::None,
            direction: ShapeDirection::Left,
            amplitude: 0.5,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FixtureSnapshot {
    pub slot: u8,
    pub id: String,
    pub label: String,
    pub selected: bool,
    pub pan: f64,
    pub tilt: f64,
    pub dimmer: f64,
    pub color: [f64; 3],
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OutputStatus {
    pub armed: bool,
    pub blackout: bool,
    pub playing: bool,
    pub target: String,
    pub port_address: u16,
    pub sequence: u8,
    pub sent_frames: u64,
    pub last_error: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FrameSnapshot {
    pub frame_id: u64,
    pub fixtures: Vec<FixtureSnapshot>,
    pub dmx: Vec<u8>,
    pub output: OutputStatus,
    pub active_page: u8,
    pub channel_bank: u8,
    pub active_group: u8,
    pub encoder_mode: EncoderMode,
    pub shape_kind: String,
    pub shape_playing: bool,
    pub master: u16,
    pub shape_params: ShapeParams,
    pub playback_levels: [u16; 16],
    pub recorded_slots: [bool; 16],
    pub programmer_active: bool,
    pub channel_values: [u8; 32],
}

#[derive(Clone, Debug)]
pub struct OutputStatusCore {
    pub armed: bool,
    pub target: String,
    pub port_address: u16,
    pub compat_zero: bool,
    pub sequence: u8,
    pub sent_frames: u64,
    pub last_error: Option<String>,
}

impl Default for OutputStatusCore {
    fn default() -> Self {
        Self {
            armed: false,
            target: String::new(),
            port_address: 0,
            compat_zero: false,
            sequence: 1,
            sent_frames: 0,
            last_error: None,
        }
    }
}

impl OutputStatusCore {
    pub fn to_api(&self, blackout: bool, playing: bool) -> OutputStatus {
        OutputStatus {
            armed: self.armed,
            blackout,
            playing,
            target: self.target.clone(),
            port_address: self.port_address,
            sequence: self.sequence,
            sent_frames: self.sent_frames,
            last_error: self.last_error.clone(),
        }
    }
}
