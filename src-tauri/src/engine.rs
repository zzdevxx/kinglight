use std::sync::{Arc, RwLock};

use crate::model::{
    EncoderMode, FixtureSnapshot, FrameSnapshot, Interleave, OutputStatusCore, ShapeDirection,
    ShapeParams,
};
use crate::storage::{ShowDocument, StoredFixture, StoredShape, StoredSlot};
use crate::{DMX_LEN, FIXTURE_COUNT, FIXTURE_FOOTPRINT};

pub const SHAPE_KINDS: [&str; 8] = [
    "Circle",
    "CircleTwoWay",
    "Figure8",
    "Spiral",
    "Square",
    "Diamond",
    "PanSwing",
    "TiltSwing",
];

const ATTRIBUTE_COUNT: usize = 6;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Attribute {
    Dimmer,
    Pan,
    Tilt,
    Red,
    Green,
    Blue,
}

impl Attribute {
    pub fn parse(value: &str) -> Option<Self> {
        match value.to_ascii_lowercase().replace('_', "").as_str() {
            "dimmer" | "intensity" => Some(Self::Dimmer),
            "pan" | "panfine" | "pancoarse" => Some(Self::Pan),
            "tilt" | "tiltfine" | "tiltcoarse" => Some(Self::Tilt),
            "red" => Some(Self::Red),
            "green" => Some(Self::Green),
            "blue" => Some(Self::Blue),
            _ => None,
        }
    }

    pub(crate) const fn index(self) -> usize {
        match self {
            Self::Dimmer => 0,
            Self::Pan => 1,
            Self::Tilt => 2,
            Self::Red => 3,
            Self::Green => 4,
            Self::Blue => 5,
        }
    }
}

#[derive(Clone, Debug)]
pub struct FixtureState {
    pub slot: u8,
    pub id: String,
    pub label: String,
    pub selected: bool,
    pub values: [u16; ATTRIBUTE_COUNT],
    pub programmer: [Option<u16>; ATTRIBUTE_COUNT],
}

impl FixtureState {
    fn new(slot: usize) -> Self {
        let pan = ((slot - 1) as u32 * u16::MAX as u32 / (FIXTURE_COUNT - 1) as u32) as u16;
        Self {
            slot: slot as u8,
            id: format!("fixture-{slot:02}"),
            label: format!("Unit {slot:02}"),
            selected: false,
            values: [0, pan, u16::MAX / 2, u16::MAX, u16::MAX, u16::MAX],
            programmer: [None; ATTRIBUTE_COUNT],
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct SlotState {
    values: [u16; ATTRIBUTE_COUNT],
}

impl SlotState {
    fn from_fixture(fixture: &FixtureState) -> Self {
        Self {
            values: fixture.values,
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ShapeState {
    pub(crate) kind: String,
    pub(crate) params: ShapeParams,
    pub(crate) playing: bool,
    pub(crate) phase_ms: u128,
}

#[derive(Clone, Debug)]
pub(crate) struct BlackoutLatch {
    pub(crate) pan: Vec<u16>,
    pub(crate) tilt: Vec<u16>,
    pub(crate) color: Vec<[u16; 3]>,
}

#[derive(Clone, Debug)]
pub struct EngineState {
    pub(crate) fixtures: Vec<FixtureState>,
    pub(crate) pages: Vec<Vec<SlotState>>,
    pub(crate) revisions: Vec<u64>,
    pub(crate) active_page: u8,
    pub(crate) channel_bank: u8,
    pub(crate) active_group: u8,
    pub(crate) encoder_mode: EncoderMode,
    pub(crate) master: u16,
    pub(crate) blackout: bool,
    pub(crate) latch: Option<BlackoutLatch>,
    pub(crate) shape: ShapeState,
    pub(crate) playing: bool,
    pub(crate) frame_id: u64,
}

#[derive(Clone, Debug)]
pub struct FrameBundle {
    pub frame_id: u64,
    pub dmx: [u8; DMX_LEN],
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct ResolvedFixture {
    pub(crate) dimmer: u16,
    pub(crate) pan: u16,
    pub(crate) tilt: u16,
    pub(crate) color: [u16; 3],
}

pub fn htp(values: &[u16]) -> u16 {
    values.iter().copied().max().unwrap_or(0)
}

pub fn resolve_intensity(playback_values: &[u16], programmer: Option<u16>) -> u16 {
    programmer.unwrap_or_else(|| htp(playback_values))
}

pub fn scale_intensity(value: u16, master: u16) -> u16 {
    ((value as u32 * master as u32) / u16::MAX as u32) as u16
}

pub fn validate_shape_kind(kind: &str) -> Result<(), String> {
    if SHAPE_KINDS.contains(&kind) {
        Ok(())
    } else {
        Err(format!("INVALID_SHAPE: bentuk {kind} tidak dikenal"))
    }
}

pub fn validate_shape_params(params: &ShapeParams) -> Result<ShapeParams, String> {
    if !params.speed.is_finite()
        || !params.size.is_finite()
        || !params.spread.is_finite()
        || !params.amplitude.is_finite()
        || params.bpm.is_some_and(|bpm| !bpm.is_finite())
    {
        return Err("INVALID_SHAPE: parameter mengandung nilai tidak hingga".to_string());
    }
    Ok(ShapeParams {
        speed: params.speed.clamp(0.01, 8.0),
        bpm: params.bpm.filter(|bpm| *bpm > 0.0),
        size: params.size.clamp(0.0, 1.0),
        spread: params.spread.clamp(0.0, 1.0),
        repeat: params.repeat.clamp(1, 16),
        interleave: params.interleave,
        direction: params.direction,
        amplitude: params.amplitude.clamp(0.0, 1.0),
    })
}

pub fn shape_values(kind: &str, params: &ShapeParams, phase_ms: u128, index: usize) -> (u16, u16) {
    let speed = params.speed.clamp(0.01, 8.0);
    let cycle_ms = match params.bpm.filter(|bpm| *bpm > 0.0) {
        Some(bpm) => 60_000.0 / (bpm * speed),
        None => 2_000.0 / speed,
    };
    let repeat = params.repeat.max(1) as f64;
    let offset = match params.interleave {
        Interleave::None => index as f64,
        Interleave::Even => (index / 2) as f64,
        Interleave::Odd => ((index + 1) / 2) as f64,
    };
    let mut t = ((phase_ms as f64 / cycle_ms.max(1.0)) * repeat + offset * params.spread)
        .rem_euclid(1.0);

    t = match params.direction {
        ShapeDirection::Left => t,
        ShapeDirection::Right => 1.0 - t,
        ShapeDirection::Bounce => {
            if t < 0.5 {
                t * 2.0
            } else {
                (1.0 - t) * 2.0
            }
        }
        ShapeDirection::Cross => {
            if index % 2 == 1 {
                1.0 - t
            } else {
                t
            }
        }
    };

    let size = params.size.clamp(0.0, 1.0) * 0.5;
    let amplitude = params.amplitude.clamp(0.0, 1.0) * 0.5;
    let tau = std::f64::consts::TAU;
    let (pan, tilt) = match kind {
        "CircleTwoWay" => {
            let folded = if t < 0.5 { t * 2.0 } else { (1.0 - t) * 2.0 };
            let angle = tau * folded;
            (0.5 + size * angle.cos(), 0.5 + size * angle.sin())
        }
        "Figure8" => (
            0.5 + size * (tau * t).sin(),
            0.5 + size * (2.0 * tau * t).sin() * 0.75,
        ),
        "Spiral" => {
            let radius = size * t;
            let angle = 2.0 * tau * t;
            (0.5 + radius * angle.cos(), 0.5 + radius * angle.sin())
        }
        "Square" => square_path(t, size),
        "Diamond" => diamond_path(t, size),
        "PanSwing" => (0.5 + amplitude * (tau * t).sin(), 0.5),
        "TiltSwing" => (0.5, 0.5 + amplitude * (tau * t).sin()),
        _ => {
            let angle = tau * t;
            (0.5 + size * angle.cos(), 0.5 + size * angle.sin())
        }
    };
    (to_u16(pan), to_u16(tilt))
}

fn square_path(t: f64, size: f64) -> (f64, f64) {
    let segment = (t * 4.0).floor() as usize;
    let local = (t * 4.0) - segment as f64;
    match segment.min(3) {
        0 => (0.5 + size, 0.5 - size + local * size * 2.0),
        1 => (0.5 + size - local * size * 2.0, 0.5 + size),
        2 => (0.5 - size, 0.5 + size - local * size * 2.0),
        _ => (0.5 - size + local * size * 2.0, 0.5 - size),
    }
}

fn diamond_path(t: f64, size: f64) -> (f64, f64) {
    let segment = (t * 4.0).floor() as usize;
    let local = (t * 4.0) - segment as f64;
    match segment.min(3) {
        0 => (0.5 + local * size, 0.5 - size + local * size),
        1 => (0.5 + size - local * size, 0.5 + local * size),
        2 => (0.5 - local * size, 0.5 + size - local * size),
        _ => (0.5 - size + local * size, 0.5 - local * size),
    }
}

fn to_u16(value: f64) -> u16 {
    (value.clamp(0.0, 1.0) * u16::MAX as f64).round() as u16
}

impl EngineState {
    pub fn new() -> Self {
        let fixtures: Vec<FixtureState> = (1..=FIXTURE_COUNT).map(FixtureState::new).collect();
        let pages = (0..6)
            .map(|page| {
                fixtures
                    .iter()
                    .map(|fixture| {
                        let mut slot = SlotState::from_fixture(fixture);
                        slot.values[0] = (((page + 1) as u32 * u16::MAX as u32 / 6) / 4) as u16;
                        slot
                    })
                    .collect()
            })
            .collect();
        Self {
            revisions: vec![1; 6],
            fixtures,
            pages,
            active_page: 1,
            channel_bank: 1,
            active_group: 0,
            encoder_mode: EncoderMode::Normal,
            master: u16::MAX,
            blackout: false,
            latch: None,
            shape: ShapeState {
                kind: "Circle".to_string(),
                params: ShapeParams::default(),
                playing: false,
                phase_ms: 0,
            },
            playing: false,
            frame_id: 0,
        }
    }

    pub(crate) fn target_indices(&self) -> Vec<usize> {
        if self.active_group > 0 {
            let start = (self.active_group as usize - 1) * 4;
            return (start..start + 4).collect();
        }
        let selected: Vec<usize> = self
            .fixtures
            .iter()
            .enumerate()
            .filter_map(|(index, fixture)| fixture.selected.then_some(index))
            .collect();
        if selected.is_empty() {
            (0..FIXTURE_COUNT).collect()
        } else {
            selected
        }
    }

    pub(crate) fn set_programmer(&mut self, indices: &[usize], attribute: Attribute, value: u16) {
        for &index in indices {
            self.fixtures[index].programmer[attribute.index()] = Some(value);
        }
    }

    pub(crate) fn set_fine_byte(&mut self, indices: &[usize], attribute: Attribute, high: bool, value: u16) {
        for &index in indices {
            let current = self.fixtures[index].values[attribute.index()];
            let updated = if high {
                (value << 8) | (current & 0x00ff)
            } else {
                (current & 0xff00) | (value & 0x00ff)
            };
            self.fixtures[index].programmer[attribute.index()] = Some(updated);
        }
    }

    pub(crate) fn resolved_values(&self, force_blackout: bool) -> Vec<ResolvedFixture> {
        let page_index = self.active_page.saturating_sub(1) as usize;
        let mut resolved = Vec::with_capacity(FIXTURE_COUNT);

        for (index, fixture) in self.fixtures.iter().enumerate() {
            let mut dimmer = fixture.values[0];
            let mut pan = fixture.values[1];
            let mut tilt = fixture.values[2];
            let mut color = [
                fixture.values[3],
                fixture.values[4],
                fixture.values[5],
            ];

            if self.playing {
                let dimmers: Vec<u16> = self.pages.iter().map(|page| page[index].values[0]).collect();
                dimmer = resolve_intensity(&dimmers, None);
                for (channel, color_index) in [3usize, 4, 5].into_iter().enumerate() {
                    let values: Vec<u16> = self
                        .pages
                        .iter()
                        .map(|page| page[index].values[color_index])
                        .collect();
                    color[channel] = htp(&values);
                }
                let ltp_page = self
                    .revisions
                    .iter()
                    .enumerate()
                    .max_by_key(|(_, revision)| **revision)
                    .map(|(page, _)| page)
                    .unwrap_or(page_index);
                pan = self.pages[ltp_page][index].values[1];
                tilt = self.pages[ltp_page][index].values[2];
            }

            if let Some(value) = fixture.programmer[Attribute::Dimmer.index()] {
                dimmer = value;
            }
            if let Some(value) = fixture.programmer[Attribute::Pan.index()] {
                pan = value;
            }
            if let Some(value) = fixture.programmer[Attribute::Tilt.index()] {
                tilt = value;
            }
            for (channel, attribute) in [
                Attribute::Red,
                Attribute::Green,
                Attribute::Blue,
            ]
            .into_iter()
            .enumerate()
            {
                if let Some(value) = fixture.programmer[attribute.index()] {
                    color[channel] = value;
                }
            }

            if self.shape.playing {
                let (shape_pan, shape_tilt) =
                    shape_values(&self.shape.kind, &self.shape.params, self.shape.phase_ms, index);
                if fixture.programmer[Attribute::Pan.index()].is_none() {
                    pan = shape_pan;
                }
                if fixture.programmer[Attribute::Tilt.index()].is_none() {
                    tilt = shape_tilt;
                }
            }

            resolved.push(ResolvedFixture {
                dimmer,
                pan,
                tilt,
                color,
            });
        }

        if self.blackout || force_blackout {
            for (index, value) in resolved.iter_mut().enumerate() {
                if let Some(latch) = &self.latch {
                    value.pan = latch.pan[index];
                    value.tilt = latch.tilt[index];
                    value.color = latch.color[index];
                }
                value.dimmer = 0;
            }
        }

        for value in &mut resolved {
            value.dimmer = scale_intensity(value.dimmer, self.master);
        }

        resolved
    }

    fn build_frame(&self, force_blackout: bool) -> FrameBundle {
        let values = self.resolved_values(force_blackout);
        let mut dmx = [0u8; DMX_LEN];
        for (index, value) in values.iter().enumerate() {
            let base = index * FIXTURE_FOOTPRINT;
            dmx[base] = (value.dimmer >> 8) as u8;
            dmx[base + 1] = (value.pan >> 8) as u8;
            dmx[base + 2] = (value.pan & 0x00ff) as u8;
            dmx[base + 3] = (value.tilt >> 8) as u8;
            dmx[base + 4] = (value.tilt & 0x00ff) as u8;
            dmx[base + 5] = (value.color[0] >> 8) as u8;
            dmx[base + 6] = (value.color[1] >> 8) as u8;
            dmx[base + 7] = (value.color[2] >> 8) as u8;
        }
        FrameBundle {
            frame_id: self.frame_id,
            dmx,
        }
    }

    fn snapshot(&self, output: &OutputStatusCore, bundle: &FrameBundle) -> FrameSnapshot {
        let values = self.resolved_values(false);
        let fixtures = values
            .into_iter()
            .enumerate()
            .map(|(index, value)| FixtureSnapshot {
                slot: self.fixtures[index].slot,
                id: self.fixtures[index].id.clone(),
                label: self.fixtures[index].label.clone(),
                selected: self.fixtures[index].selected,
                pan: value.pan as f64 / u16::MAX as f64,
                tilt: value.tilt as f64 / u16::MAX as f64,
                dimmer: value.dimmer as f64 / u16::MAX as f64,
                color: value.color.map(|channel| channel as f64 / u16::MAX as f64),
            })
            .collect();
        FrameSnapshot {
            frame_id: bundle.frame_id,
            fixtures,
            dmx: bundle.dmx.to_vec(),
            output: output.to_api(self.blackout, self.playing),
            active_page: self.active_page,
            channel_bank: self.channel_bank,
            active_group: self.active_group,
            encoder_mode: self.encoder_mode,
            shape_kind: self.shape.kind.clone(),
            shape_playing: self.shape.playing,
        }
    }

    pub fn to_document(&self) -> ShowDocument {
        ShowDocument {
            schema_version: 1,
            show_id: "kinglight-local-show".to_string(),
            name: "Kinglight Show".to_string(),
            profile_digest: crate::storage::PROFILE_DIGEST.to_string(),
            master: self.master,
            fixtures: self
                .fixtures
                .iter()
                .map(|fixture| StoredFixture {
                    slot: fixture.slot,
                    id: fixture.id.clone(),
                    label: fixture.label.clone(),
                    dimmer: fixture.values[0],
                    pan: fixture.values[1],
                    tilt: fixture.values[2],
                    color: [fixture.values[3], fixture.values[4], fixture.values[5]],
                })
                .collect(),
            pages: self
                .pages
                .iter()
                .map(|page| {
                    page.iter()
                        .map(|slot| StoredSlot {
                            dimmer: slot.values[0],
                            pan: slot.values[1],
                            tilt: slot.values[2],
                            color: [slot.values[3], slot.values[4], slot.values[5]],
                        })
                        .collect()
                })
                .collect(),
            shape: StoredShape {
                kind: self.shape.kind.clone(),
                params: self.shape.params.clone(),
            },
        }
    }

    pub fn replace_from_document(document: &ShowDocument) -> Result<Self, String> {
        let mut state = Self::new();
        if document.fixtures.len() != FIXTURE_COUNT {
            return Err("INVALID_SHOW: jumlah fixture bukan 16".to_string());
        }
        for (index, stored) in document.fixtures.iter().enumerate() {
            if stored.slot as usize != index + 1 {
                return Err("INVALID_SHOW: nomor slot fixture tidak berurutan".to_string());
            }
            let fixture = &mut state.fixtures[index];
            fixture.id = stored.id.clone();
            fixture.label = stored.label.clone();
            fixture.values = [
                stored.dimmer,
                stored.pan,
                stored.tilt,
                stored.color[0],
                stored.color[1],
                stored.color[2],
            ];
            fixture.programmer = [None; ATTRIBUTE_COUNT];
        }
        if document.pages.len() != 6 {
            return Err("INVALID_SHOW: jumlah page bukan 6".to_string());
        }
        for (page_index, page) in document.pages.iter().enumerate() {
            if page.len() != FIXTURE_COUNT {
                return Err("INVALID_SHOW: jumlah slot page bukan 16".to_string());
            }
            for (slot_index, stored) in page.iter().enumerate() {
                state.pages[page_index][slot_index].values = [
                    stored.dimmer,
                    stored.pan,
                    stored.tilt,
                    stored.color[0],
                    stored.color[1],
                    stored.color[2],
                ];
            }
        }
        validate_shape_kind(&document.shape.kind)?;
        state.shape.kind = document.shape.kind.clone();
        state.shape.params = validate_shape_params(&document.shape.params)?;
        state.shape.playing = false;
        state.shape.phase_ms = 0;
        state.master = document.master;
        state.playing = false;
        state.blackout = false;
        state.latch = None;
        state.frame_id = 0;
        Ok(state)
    }
}

pub struct Engine {
    state: RwLock<EngineState>,
    latest: RwLock<Arc<FrameBundle>>,
}

impl Engine {
    pub fn new() -> Self {
        let state = EngineState::new();
        let bundle = Arc::new(state.build_frame(false));
        Self {
            state: RwLock::new(state),
            latest: RwLock::new(bundle),
        }
    }

    pub fn advance(&self, delta_ms: u128) {
        let mut state = match self.state.write() {
            Ok(guard) => guard,
            Err(error) => {
                eprintln!("[kinglight] error=engine_state_poisoned detail={error}");
                return;
            }
        };
        if state.shape.playing {
            state.shape.phase_ms = state.shape.phase_ms.saturating_add(delta_ms);
        }
        state.frame_id = state.frame_id.wrapping_add(1);
        let bundle = Arc::new(state.build_frame(false));
        drop(state);
        self.publish_bundle(bundle);
    }

    fn publish_bundle(&self, bundle: Arc<FrameBundle>) {
        let mut latest = match self.latest.write() {
            Ok(guard) => guard,
            Err(error) => {
                eprintln!("[kinglight] error=engine_frame_poisoned detail={error}");
                return;
            }
        };
        *latest = bundle;
    }

    fn current_bundle(&self, state: &EngineState) -> Arc<FrameBundle> {
        Arc::new(state.build_frame(false))
    }

    pub fn mutate<F>(&self, output: &OutputStatusCore, mutate: F) -> Result<FrameSnapshot, String>
    where
        F: FnOnce(&mut EngineState) -> Result<(), String>,
    {
        let mut state = self.state.write().map_err(|error| {
            eprintln!("[kinglight] error=engine_state_poisoned detail={error}");
            "ENGINE_STATE_CORRUPT".to_string()
        })?;
        mutate(&mut state)?;
        state.frame_id = state.frame_id.wrapping_add(1);
        let bundle = self.current_bundle(&state);
        self.publish_bundle(bundle.clone());
        Ok(state.snapshot(output, &bundle))
    }

    pub fn snapshot(&self, output: &OutputStatusCore) -> FrameSnapshot {
        let state = match self.state.read() {
            Ok(guard) => guard,
            Err(error) => {
                eprintln!("[kinglight] error=engine_state_poisoned detail={error}");
                return FrameSnapshot {
                    frame_id: 0,
                    fixtures: Vec::new(),
                    dmx: vec![0; DMX_LEN],
                    output: output.to_api(false, false),
                    active_page: 1,
                    channel_bank: 1,
                    active_group: 0,
                    encoder_mode: EncoderMode::Normal,
                    shape_kind: SHAPE_KINDS[0].to_string(),
                    shape_playing: false,
                };
            }
        };
        let bundle = self.current_bundle(&state);
        state.snapshot(output, &bundle)
    }

    pub fn document(&self) -> ShowDocument {
        match self.state.read() {
            Ok(guard) => guard.to_document(),
            Err(error) => {
                eprintln!("[kinglight] error=engine_state_poisoned detail={error}");
                EngineState::new().to_document()
            }
        }
    }

    pub fn frame(&self) -> Arc<FrameBundle> {
        match self.latest.read() {
            Ok(guard) => guard.clone(),
            Err(error) => {
                eprintln!("[kinglight] error=engine_frame_poisoned detail={error}");
                Arc::new(FrameBundle {
                    frame_id: 0,
                    dmx: [0; DMX_LEN],
                })
            }
        }
    }

    pub fn stop_frame(&self) -> FrameBundle {
        let state = match self.state.read() {
            Ok(guard) => guard,
            Err(error) => {
                eprintln!("[kinglight] error=engine_state_poisoned detail={error}");
                return FrameBundle {
                    frame_id: 0,
                    dmx: [0; DMX_LEN],
                };
            }
        };
        state.build_frame(true)
    }

    pub fn replace_show(&self, document: &ShowDocument) -> Result<(), String> {
        let state = EngineState::replace_from_document(document)?;
        let bundle = Arc::new(state.build_frame(false));
        let mut guard = self.state.write().map_err(|error| {
            eprintln!("[kinglight] error=engine_state_poisoned detail={error}");
            "ENGINE_STATE_CORRUPT".to_string()
        })?;
        *guard = state;
        drop(guard);
        self.publish_bundle(bundle);
        Ok(())
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn htp_selects_highest_playback_and_programmer_overrides_absolutely() {
        let playback = [120u16, 80u16, 200u16];
        assert_eq!(htp(&playback), 200);
        assert_eq!(resolve_intensity(&playback, None), 200);
        assert_eq!(resolve_intensity(&playback, Some(80)), 80);
    }

    #[test]
    fn programmer_override_applies_only_to_masked_attributes() {
        let mut state = EngineState::new();
        state.playing = true;
        state.shape.playing = true;
        let indices = vec![0usize];
        state.set_programmer(&indices, Attribute::Pan, 1234);
        let resolved = state.resolved_values(false);
        assert_eq!(resolved[0].pan, 1234);
        assert_ne!(resolved[0].tilt, 1234);
    }

    #[test]
    fn blackout_latch_holds_pan_tilt_and_color_while_phase_advances() {
        let mut state = EngineState::new();
        state.shape.playing = true;
        let before = state.resolved_values(false);
        state.blackout = true;
        state.latch = Some(BlackoutLatch {
            pan: before.iter().map(|value| value.pan).collect(),
            tilt: before.iter().map(|value| value.tilt).collect(),
            color: before.iter().map(|value| value.color).collect(),
        });
        state.shape.phase_ms += 1_000;
        let during = state.resolved_values(false);
        assert_eq!(during[0].pan, before[0].pan);
        assert_eq!(during[0].tilt, before[0].tilt);
        assert_eq!(during[0].color, before[0].color);
        assert_eq!(during[0].dimmer, 0);
        assert_ne!(shape_values("Circle", &state.shape.params, 0, 0), shape_values("Circle", &state.shape.params, 1_000, 0));
    }

    #[test]
    fn shape_values_are_deterministic_and_distinct_per_kind() {
        let params = ShapeParams::default();
        let first = shape_values("Figure8", &params, 750, 3);
        let second = shape_values("Figure8", &params, 750, 3);
        assert_eq!(first, second);
        assert_ne!(first, shape_values("Spiral", &params, 750, 3));
    }

    #[test]
    fn master_scales_intensity_without_changing_pan_or_tilt() {
        let mut state = EngineState::new();
        state.playing = true;
        let before = state.resolved_values(false);
        state.master = u16::MAX / 2;
        let after = state.resolved_values(false);
        assert_eq!(after[0].pan, before[0].pan);
        assert_eq!(after[0].tilt, before[0].tilt);
        assert_eq!(after[0].color, before[0].color);
        assert!(after[0].dimmer < before[0].dimmer);
    }

    #[test]
    fn frame_has_512_bytes_and_only_first_128_channels_are_written() {
        let state = EngineState::new();
        let frame = state.build_frame(false);
        assert_eq!(frame.dmx.len(), 512);
        assert!(frame.dmx[128..].iter().all(|byte| *byte == 0));
    }

    #[test]
    fn shape_kind_is_validated_against_the_eight_names() {
        assert!(validate_shape_kind("Diamond").is_ok());
        assert!(validate_shape_kind("Triangle").is_err());
    }
}
