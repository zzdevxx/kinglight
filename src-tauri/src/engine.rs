use std::sync::{Arc, RwLock};

use crate::model::{
    EncoderMode, FixtureSnapshot, FrameSnapshot, Interleave, OutputStatusCore, ShapeDirection,
    ShapeParams,
};
use crate::storage::{
    ShowDocument, StoredCueFixture, StoredFixture, StoredProgrammer, StoredShape, StoredSlot,
};
use crate::{DMX_LEN, FIXTURE_COUNT, FIXTURE_FOOTPRINT};

pub const SHAPE_KINDS: [&str; 11] = [
    "Circle",
    "CircleTwoWay",
    "Figure8",
    "Spiral",
    "Square",
    "Diamond",
    "PanSwing",
    "TiltSwing",
    "Dimmer",
    "RGB",
    "Color",
];

const ATTRIBUTE_COUNT: usize = 6;
const PAGE_COUNT: usize = 6;
const SLOT_COUNT: usize = 16;
const CHANNEL_FADER_COUNT: u16 = 32;
const VISIBLE_FIXTURE_COUNT: usize = 12;

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ChannelAttribute {
    Value(Attribute),
    High(Attribute),
    Low(Attribute),
}

fn channel_attribute(offset: usize) -> ChannelAttribute {
    match offset {
        0 => ChannelAttribute::Value(Attribute::Dimmer),
        1 => ChannelAttribute::High(Attribute::Pan),
        2 => ChannelAttribute::Low(Attribute::Pan),
        3 => ChannelAttribute::High(Attribute::Tilt),
        4 => ChannelAttribute::Low(Attribute::Tilt),
        5 => ChannelAttribute::Value(Attribute::Red),
        6 => ChannelAttribute::Value(Attribute::Green),
        _ => ChannelAttribute::Value(Attribute::Blue),
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
        let center = u16::MAX / 2;
        let (dimmer, pan) = if slot <= VISIBLE_FIXTURE_COUNT {
            let spread = (slot - 1) as f64 / (VISIBLE_FIXTURE_COUNT - 1) as f64;
            (u16::MAX, to_u16(0.08 + 0.84 * spread))
        } else {
            (0, center)
        };
        Self {
            slot: slot as u8,
            id: format!("fixture-{slot:02}"),
            label: format!("Unit {slot:02}"),
            selected: false,
            values: [dimmer, pan, center, u16::MAX, u16::MAX, u16::MAX],
            programmer: [None; ATTRIBUTE_COUNT],
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct SlotState {
    values: [u16; ATTRIBUTE_COUNT],
    fixtures: Vec<[u16; ATTRIBUTE_COUNT]>,
}

impl SlotState {
    fn from_fixtures(fixtures: &[FixtureState]) -> Self {
        let captured: Vec<[u16; ATTRIBUTE_COUNT]> = fixtures.iter().map(|item| item.values).collect();
        Self {
            values: captured[0],
            fixtures: captured,
        }
    }

    fn blank() -> Self {
        let center = u16::MAX / 2;
        let values = [0, center, center, u16::MAX, u16::MAX, u16::MAX];
        Self {
            values,
            fixtures: vec![values; FIXTURE_COUNT],
        }
    }

    fn values_for(&self, index: usize) -> [u16; ATTRIBUTE_COUNT] {
        self.fixtures
            .get(index)
            .copied()
            .unwrap_or(self.values)
    }

    fn replace(&mut self, fixtures: Vec<[u16; ATTRIBUTE_COUNT]>) {
        self.values = fixtures.first().copied().unwrap_or_else(Self::blank_template);
        self.fixtures = fixtures;
    }

    fn blank_template() -> [u16; ATTRIBUTE_COUNT] {
        let center = u16::MAX / 2;
        [0, center, center, u16::MAX, u16::MAX, u16::MAX]
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ShapeState {
    pub(crate) kind: String,
    pub(crate) params: ShapeParams,
    pub(crate) playing: bool,
    pub(crate) phase_ms: u128,
    pub(crate) targets: Vec<usize>,
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
    pub(crate) levels: Vec<Vec<u16>>,
    pub(crate) recorded: Vec<Vec<bool>>,
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

#[derive(Clone, Copy, Debug, Default)]
pub struct ShapeEffect {
    pub(crate) dimmer: Option<u16>,
    pub(crate) pan: Option<u16>,
    pub(crate) tilt: Option<u16>,
    pub(crate) color: Option<[u16; 3]>,
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

pub fn encoder_step(mode: EncoderMode) -> i64 {
    match mode {
        EncoderMode::Normal => 1024,
        EncoderMode::Fine => 128,
        EncoderMode::Ultra => 16,
    }
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
        repeat: params.repeat.clamp(0, 16),
        interleave: params.interleave,
        direction: params.direction,
        amplitude: params.amplitude.clamp(0.0, 1.0),
    })
}

fn shape_phase(
    params: &ShapeParams,
    phase_ms: u128,
    position: usize,
    target_count: usize,
) -> f64 {
    let cycle_ms = match params.bpm.filter(|bpm| *bpm > 0.0) {
        Some(bpm) => 60_000.0 / bpm,
        None => 2_000.0 / params.speed.clamp(0.01, 8.0),
    };
    let temporal = phase_ms as f64 / cycle_ms.max(1.0);
    let repeats = params.repeat.clamp(0, 16) as f64;
    let positioned = match params.interleave {
        Interleave::None => position as f64,
        Interleave::Even => (position / 2) as f64,
        Interleave::Odd => ((position + 1) / 2) as f64,
    };
    let spatial = (positioned / target_count.max(1) as f64)
        * params.spread.clamp(0.0, 1.0)
        * (repeats / 16.0);
    let mut t = (temporal + spatial).rem_euclid(1.0);
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
            if position % 2 == 1 {
                1.0 - t
            } else {
                t
            }
        }
    };
    t
}

pub fn shape_values_for(
    kind: &str,
    params: &ShapeParams,
    phase_ms: u128,
    position: usize,
    target_count: usize,
) -> (u16, u16) {
    let t = shape_phase(params, phase_ms, position, target_count);
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

pub fn shape_values(kind: &str, params: &ShapeParams, phase_ms: u128, index: usize) -> (u16, u16) {
    shape_values_for(kind, params, phase_ms, index, FIXTURE_COUNT)
}

pub fn shape_effect(
    kind: &str,
    params: &ShapeParams,
    phase_ms: u128,
    position: usize,
    target_count: usize,
) -> ShapeEffect {
    let t = shape_phase(params, phase_ms, position, target_count);
    let tau = std::f64::consts::TAU;
    match kind {
        "Dimmer" => {
            let wave = 0.5 + params.amplitude.clamp(0.0, 1.0) * 0.5 * (tau * t).sin();
            ShapeEffect {
                dimmer: Some(to_u16(wave)),
                ..ShapeEffect::default()
            }
        }
        "RGB" => {
            let wave = |phase: f64| to_u16(0.5 + 0.5 * (tau * t + phase).sin());
            ShapeEffect {
                dimmer: Some(u16::MAX),
                color: Some([wave(0.0), wave(tau / 3.0), wave(2.0 * tau / 3.0)]),
                ..ShapeEffect::default()
            }
        }
        "Color" => {
            const PALETTE: [[u16; 3]; 6] = [
                [u16::MAX, u16::MAX, u16::MAX],
                [u16::MAX, 0, 0],
                [0, u16::MAX, 0],
                [0, 0, u16::MAX],
                [u16::MAX, u16::MAX, 0],
                [0, u16::MAX, u16::MAX],
            ];
            let step = ((t * PALETTE.len() as f64).floor() as usize) % PALETTE.len();
            ShapeEffect {
                dimmer: Some(u16::MAX),
                color: Some(PALETTE[step]),
                ..ShapeEffect::default()
            }
        }
        _ => {
            let (pan, tilt) = shape_values_for(kind, params, phase_ms, position, target_count);
            ShapeEffect {
                pan: Some(pan),
                tilt: Some(tilt),
                ..ShapeEffect::default()
            }
        }
    }
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

fn resolved_to_values(value: ResolvedFixture) -> [u16; ATTRIBUTE_COUNT] {
    [
        value.dimmer,
        value.pan,
        value.tilt,
        value.color[0],
        value.color[1],
        value.color[2],
    ]
}

fn values_to_resolved(values: [u16; ATTRIBUTE_COUNT]) -> ResolvedFixture {
    ResolvedFixture {
        dimmer: values[0],
        pan: values[1],
        tilt: values[2],
        color: [values[3], values[4], values[5]],
    }
}

fn validate_slot(slot: u8) -> Result<usize, String> {
    if slot == 0 || slot as usize > SLOT_COUNT {
        return Err(format!("INVALID_SLOT: slot {slot} di luar rentang 1..16"));
    }
    Ok(slot as usize - 1)
}

impl EngineState {
    pub fn new() -> Self {
        let fixtures: Vec<FixtureState> = (1..=FIXTURE_COUNT).map(FixtureState::new).collect();
        let pages = (0..PAGE_COUNT)
            .map(|_| vec![SlotState::from_fixtures(&fixtures); SLOT_COUNT])
            .collect();
        Self {
            levels: vec![vec![0u16; SLOT_COUNT]; PAGE_COUNT],
            recorded: vec![vec![false; SLOT_COUNT]; PAGE_COUNT],
            revisions: vec![1; PAGE_COUNT],
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
                kind: SHAPE_KINDS[0].to_string(),
                params: ShapeParams::default(),
                playing: false,
                phase_ms: 0,
                targets: (0..FIXTURE_COUNT).collect(),
            },
            playing: false,
            frame_id: 0,
        }
    }

    fn page_index(&self) -> usize {
        self.active_page.saturating_sub(1) as usize
    }

    pub(crate) fn selected_indices(&self) -> Vec<usize> {
        self.fixtures
            .iter()
            .enumerate()
            .filter_map(|(index, fixture)| fixture.selected.then_some(index))
            .collect()
    }

    pub(crate) fn target_indices(&self) -> Vec<usize> {
        if self.active_group > 0 {
            let start = (self.active_group as usize - 1) * 4;
            return (start..start + 4).collect();
        }
        let selected = self.selected_indices();
        if selected.is_empty() {
            (0..FIXTURE_COUNT).collect()
        } else {
            selected
        }
    }

    fn shape_targets(&self) -> Vec<usize> {
        let selected = self.selected_indices();
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

    pub(crate) fn set_fine_byte(
        &mut self,
        indices: &[usize],
        attribute: Attribute,
        high: bool,
        value: u16,
    ) {
        let byte = if high {
            (value >> 8) as u8
        } else {
            (value & 0x00ff) as u8
        };
        for &index in indices {
            let fixture = &mut self.fixtures[index];
            let current = fixture.programmer[attribute.index()]
                .unwrap_or(fixture.values[attribute.index()]);
            let updated = if high {
                (u16::from(byte) << 8) | (current & 0x00ff)
            } else {
                (current & 0xff00) | u16::from(byte)
            };
            fixture.programmer[attribute.index()] = Some(updated);
        }
    }

    fn playback_layer(&self, index: usize) -> Option<ResolvedFixture> {
        let mut dimmer = 0u16;
        let mut color = [0u16; 3];
        let mut best: Option<((u64, u64, u64), usize, usize)> = None;
        for page in 0..PAGE_COUNT {
            for slot in 0..SLOT_COUNT {
                let level = self.levels[page][slot];
                if level == 0 || !self.recorded[page][slot] {
                    continue;
                }
                let stored = self.pages[page][slot].values_for(index);
                dimmer = dimmer.max(scale_intensity(stored[0], level));
                for channel in 0..3 {
                    color[channel] = color[channel].max(scale_intensity(stored[3 + channel], level));
                }
                let priority = (self.revisions[page], u64::from(level), (SLOT_COUNT - slot) as u64);
                if best.as_ref().is_none_or(|(current, _, _)| priority > *current) {
                    best = Some((priority, page, slot));
                }
            }
        }
        best.map(|(_, page, slot)| {
            let mut values = values_to_resolved(self.pages[page][slot].values_for(index));
            values.dimmer = dimmer;
            values.color = color;
            values
        })
    }

    fn resolved_ex(&self, force_blackout: bool, apply_master: bool) -> Vec<ResolvedFixture> {
        let mut resolved = Vec::with_capacity(FIXTURE_COUNT);
        for (index, fixture) in self.fixtures.iter().enumerate() {
            let mut value = ResolvedFixture {
                dimmer: fixture.values[0],
                pan: fixture.values[1],
                tilt: fixture.values[2],
                color: [fixture.values[3], fixture.values[4], fixture.values[5]],
            };

            if self.playing {
                if let Some(playback) = self.playback_layer(index) {
                    value = playback;
                }
            }

            if let Some(programmer) = fixture.programmer[Attribute::Dimmer.index()] {
                value.dimmer = programmer;
            }
            if let Some(programmer) = fixture.programmer[Attribute::Pan.index()] {
                value.pan = programmer;
            }
            if let Some(programmer) = fixture.programmer[Attribute::Tilt.index()] {
                value.tilt = programmer;
            }
            for (channel, attribute) in [
                Attribute::Red,
                Attribute::Green,
                Attribute::Blue,
            ]
            .into_iter()
            .enumerate()
            {
                if let Some(programmer) = fixture.programmer[attribute.index()] {
                    value.color[channel] = programmer;
                }
            }

            if self.shape.playing {
                let target = self
                    .shape
                    .targets
                    .iter()
                    .position(|&candidate| candidate == index);
                if let Some(position) = target {
                    let effect = shape_effect(
                        &self.shape.kind,
                        &self.shape.params,
                        self.shape.phase_ms,
                        position,
                        self.shape.targets.len(),
                    );
                    if fixture.programmer[Attribute::Dimmer.index()].is_none() {
                        if let Some(dimmer) = effect.dimmer {
                            value.dimmer = dimmer;
                        }
                    }
                    if fixture.programmer[Attribute::Pan.index()].is_none() {
                        if let Some(pan) = effect.pan {
                            value.pan = pan;
                        }
                    }
                    if fixture.programmer[Attribute::Tilt.index()].is_none() {
                        if let Some(tilt) = effect.tilt {
                            value.tilt = tilt;
                        }
                    }
                    if fixture.programmer[Attribute::Red.index()].is_none()
                        && fixture.programmer[Attribute::Green.index()].is_none()
                        && fixture.programmer[Attribute::Blue.index()].is_none()
                    {
                        if let Some(color) = effect.color {
                            value.color = color;
                        }
                    }
                }
            }

            resolved.push(value);
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

        if apply_master {
            for value in &mut resolved {
                value.dimmer = scale_intensity(value.dimmer, self.master);
            }
        }
        resolved
    }

    pub(crate) fn resolved_values(&self, force_blackout: bool) -> Vec<ResolvedFixture> {
        self.resolved_ex(force_blackout, true)
    }

    fn logical_values(&self) -> Vec<ResolvedFixture> {
        self.resolved_ex(false, false)
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
        let first_selected = self.fixtures.iter().position(|fixture| fixture.selected);
        let base = first_selected
            .map(|index| index * FIXTURE_FOOTPRINT)
            .unwrap_or(0);
        let mut channel_values = [0u8; 32];
        for (offset, value) in channel_values.iter_mut().enumerate() {
            *value = bundle.dmx.get(base + offset).copied().unwrap_or(0);
        }
        let page = self.page_index();
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
            master: self.master,
            shape_params: self.shape.params.clone(),
            playback_levels: std::array::from_fn(|slot| self.levels[page][slot]),
            recorded_slots: std::array::from_fn(|slot| self.recorded[page][slot]),
            programmer_active: self
                .fixtures
                .iter()
                .any(|fixture| fixture.programmer.iter().any(Option::is_some)),
            channel_values,
        }
    }

    pub fn toggle_fixture(&mut self, slot: u8) -> Result<(), String> {
        let index = validate_slot(slot)?;
        if self.active_group > 0 {
            for fixture in &mut self.fixtures {
                fixture.selected = false;
            }
            self.active_group = 0;
        }
        self.fixtures[index].selected = !self.fixtures[index].selected;
        Ok(())
    }

    pub fn set_group(&mut self, group: u8) -> Result<(), String> {
        if group > 4 {
            return Err("INVALID_GROUP: group harus 0 sampai 4".to_string());
        }
        self.active_group = group;
        for fixture in &mut self.fixtures {
            fixture.selected = false;
        }
        if group > 0 {
            let start = (group as usize - 1) * 4;
            for fixture in &mut self.fixtures[start..start + 4] {
                fixture.selected = true;
            }
        }
        Ok(())
    }

    pub fn set_page(&mut self, page: u8) -> Result<(), String> {
        if page == 0 || page as usize > PAGE_COUNT {
            return Err("INVALID_PAGE: page harus 1 sampai 6".to_string());
        }
        self.active_page = page;
        let revision = self.revisions.iter().copied().max().unwrap_or(0) + 1;
        self.revisions[page as usize - 1] = revision;
        Ok(())
    }

    pub fn set_bank(&mut self, bank: u8) -> Result<(), String> {
        if bank == 0 || bank > 2 {
            return Err("INVALID_BANK: bank harus 1 atau 2".to_string());
        }
        self.channel_bank = bank;
        Ok(())
    }

    pub fn set_named_fader(&mut self, attribute: &str, value: u16) -> Result<(), String> {
        let normalized = attribute.to_ascii_lowercase().replace(['_', '-'], "");
        let indices = self.target_indices();
        match normalized.as_str() {
            "master" => self.master = value,
            "pancoarse" => self.set_fine_byte(&indices, Attribute::Pan, true, value),
            "panfine" => self.set_fine_byte(&indices, Attribute::Pan, false, value),
            "tiltcoarse" => self.set_fine_byte(&indices, Attribute::Tilt, true, value),
            "tiltfine" => self.set_fine_byte(&indices, Attribute::Tilt, false, value),
            name => {
                let parsed = Attribute::parse(name)
                    .ok_or_else(|| format!("INVALID_ATTRIBUTE: atribut {attribute} tidak dikenal"))?;
                self.set_programmer(&indices, parsed, value);
            }
        }
        Ok(())
    }

    pub fn set_channel_fader(&mut self, channel: u16, value: u16) -> Result<(), String> {
        if channel == 0 || channel > CHANNEL_FADER_COUNT {
            return Err(format!(
                "INVALID_CHANNEL: channel {channel} di luar rentang 1..32"
            ));
        }
        if channel as usize > FIXTURE_FOOTPRINT {
            return Err(format!(
                "INVALID_CHANNEL: channel {channel} di luar footprint fixture yang dipilih"
            ));
        }
        let index = self.selected_indices().first().copied().ok_or_else(|| {
            "INVALID_CHANNEL: pilih fixture sebelum mengatur fader channel".to_string()
        })?;
        let offset = channel as usize - 1;
        match channel_attribute(offset) {
            ChannelAttribute::Value(attribute) => self.set_programmer(&[index], attribute, value),
            ChannelAttribute::High(attribute) => {
                self.set_fine_byte(&[index], attribute, true, value)
            }
            ChannelAttribute::Low(attribute) => {
                self.set_fine_byte(&[index], attribute, false, value)
            }
        }
        Ok(())
    }

    pub fn set_fader_attribute(&mut self, attribute: &str, value: u16) -> Result<(), String> {
        let normalized: String = attribute
            .chars()
            .filter(|character| !matches!(character, '.' | '_' | '-'))
            .collect::<String>()
            .to_ascii_lowercase();
        let channel_digits = normalized
            .strip_prefix("channel")
            .or_else(|| normalized.strip_prefix("ch"));
        if let Some(digits) = channel_digits {
            if !digits.is_empty() {
                let channel = digits.parse::<u16>().map_err(|_| {
                    format!("INVALID_CHANNEL: atribut {attribute} bukan channel valid")
                })?;
                return self.set_channel_fader(channel, value);
            }
        }
        self.set_named_fader(attribute, value)
    }

    pub fn encoder_tick(
        &mut self,
        axis: &str,
        delta: i32,
        mode: EncoderMode,
    ) -> Result<(), String> {
        let normalized = axis.to_ascii_lowercase();
        if normalized != "a" && normalized != "b" {
            return Err("INVALID_ENCODER_AXIS: axis harus a atau b".to_string());
        }
        self.encoder_mode = mode;
        if delta == 0 {
            return Ok(());
        }
        let step = encoder_step(mode);
        let attribute = if normalized == "a" {
            Attribute::Pan
        } else {
            Attribute::Tilt
        };
        let indices = self.target_indices();
        for index in indices {
            let current = self.fixtures[index].programmer[attribute.index()]
                .unwrap_or(self.fixtures[index].values[attribute.index()]);
            let next = (i64::from(current) + i64::from(delta) * step).clamp(0, i64::from(u16::MAX));
            self.fixtures[index].programmer[attribute.index()] = Some(next as u16);
        }
        Ok(())
    }

    pub fn apply_shape(&mut self, kind: &str, params: ShapeParams) -> Result<(), String> {
        validate_shape_kind(kind)?;
        let params = validate_shape_params(&params)?;
        if self.shape.kind != kind {
            self.shape.phase_ms = 0;
        }
        self.shape.kind = kind.to_string();
        self.shape.params = params;
        self.shape.targets = self.shape_targets();
        Ok(())
    }

    pub fn shape_play(&mut self, active: bool) {
        if active && !self.shape.playing {
            self.shape.phase_ms = 0;
            self.shape.targets = self.shape_targets();
        }
        self.shape.playing = active;
    }

    pub fn set_blackout(&mut self, active: bool) {
        if active && !self.blackout {
            let values = self.resolved_values(false);
            self.latch = Some(BlackoutLatch {
                pan: values.iter().map(|value| value.pan).collect(),
                tilt: values.iter().map(|value| value.tilt).collect(),
                color: values.iter().map(|value| value.color).collect(),
            });
        }
        if !active {
            self.latch = None;
        }
        self.blackout = active;
    }

    pub fn set_playing(&mut self, active: bool) {
        self.playing = active;
    }

    pub fn set_playback(&mut self, slot: u8, value: u16) -> Result<(), String> {
        let index = validate_slot(slot)?;
        let page = self.page_index();
        self.levels[page][index] = value;
        if value > 0 {
            self.playing = true;
            let revision = self.revisions.iter().copied().max().unwrap_or(0) + 1;
            self.revisions[page] = revision;
        }
        Ok(())
    }

    pub fn console_action(
        &mut self,
        action: &str,
        slot: Option<u8>,
        source: Option<u8>,
    ) -> Result<(), String> {
        if let Some(slot) = slot {
            validate_slot(slot)?;
        }
        if let Some(source) = source {
            validate_slot(source)?;
        }
        let action = action.trim().to_ascii_lowercase();
        match action.as_str() {
            "clear" => {
                for fixture in &mut self.fixtures {
                    fixture.programmer = [None; ATTRIBUTE_COUNT];
                    fixture.selected = false;
                }
                self.active_group = 0;
            }
            "record" => {
                let target = require_slot(slot, &action)?;
                let captured: Vec<[u16; ATTRIBUTE_COUNT]> = self
                    .logical_values()
                    .into_iter()
                    .map(resolved_to_values)
                    .collect();
                let page = self.page_index();
                self.pages[page][target].replace(captured);
                self.recorded[page][target] = true;
            }
            "include" => {
                let target = require_slot(slot, &action)?;
                let page = self.page_index();
                if !self.recorded[page][target] {
                    return Err(format!("EMPTY_SLOT: slot {} belum direkam", target + 1));
                }
                let values: Vec<[u16; ATTRIBUTE_COUNT]> = (0..FIXTURE_COUNT)
                    .map(|index| self.pages[page][target].values_for(index))
                    .collect();
                for (index, fixture) in self.fixtures.iter_mut().enumerate() {
                    fixture.programmer = values[index].map(Some);
                }
            }
            "copy" | "move" => {
                let target = require_slot(slot, &action)?;
                let from = source.ok_or_else(|| {
                    "INVALID_SOURCE: source wajib diisi untuk copy dan move".to_string()
                })?;
                let from = validate_slot(from)?;
                let page = self.page_index();
                if action == "move" && from == target {
                    return Ok(());
                }
                if !self.recorded[page][from] {
                    return Err(format!("EMPTY_SLOT: slot {} belum direkam", from + 1));
                }
                self.pages[page][target] = self.pages[page][from].clone();
                self.recorded[page][target] = true;
                if action == "move" {
                    self.pages[page][from] = SlotState::blank();
                    self.recorded[page][from] = false;
                    self.levels[page][from] = 0;
                }
            }
            "delete" => {
                let target = require_slot(slot, &action)?;
                let page = self.page_index();
                self.pages[page][target] = SlotState::blank();
                self.recorded[page][target] = false;
                self.levels[page][target] = 0;
            }
            "select_all" => {
                self.active_group = 0;
                for fixture in &mut self.fixtures {
                    fixture.selected = true;
                }
            }
            "locate" => {
                let indices = self.target_indices();
                self.set_programmer(&indices, Attribute::Dimmer, u16::MAX);
            }
            _ => return Err(format!("INVALID_CONSOLE_ACTION: aksi {action} tidak dikenal")),
        }
        Ok(())
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
                            fixtures: Some(
                                slot.fixtures
                                    .iter()
                                    .map(|values| StoredCueFixture { values: *values })
                                    .collect(),
                            ),
                        })
                        .collect()
                })
                .collect(),
            shape: StoredShape {
                kind: self.shape.kind.clone(),
                params: self.shape.params.clone(),
            },
            programmer: Some(
                self.fixtures
                    .iter()
                    .map(|fixture| StoredProgrammer {
                        values: fixture.programmer,
                    })
                    .collect(),
            ),
            recorded_slots: Some(self.recorded.clone()),
            playback_levels: Some(self.levels.clone()),
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
        if document.pages.len() != PAGE_COUNT {
            return Err("INVALID_SHOW: jumlah page bukan 6".to_string());
        }
        for (page_index, page) in document.pages.iter().enumerate() {
            if page.len() != SLOT_COUNT {
                return Err("INVALID_SHOW: jumlah slot page bukan 16".to_string());
            }
            for (slot_index, stored) in page.iter().enumerate() {
                let template = [
                    stored.dimmer,
                    stored.pan,
                    stored.tilt,
                    stored.color[0],
                    stored.color[1],
                    stored.color[2],
                ];
                let fixtures = match &stored.fixtures {
                    Some(fixtures) => fixtures
                        .iter()
                        .map(|fixture| fixture.values)
                        .collect::<Vec<[u16; ATTRIBUTE_COUNT]>>(),
                    None => vec![template; FIXTURE_COUNT],
                };
                state.pages[page_index][slot_index].replace(fixtures);
            }
        }
        if let Some(programmer) = &document.programmer {
            if programmer.len() != FIXTURE_COUNT {
                return Err("INVALID_SHOW: jumlah programmer fixture bukan 16".to_string());
            }
            for (index, stored) in programmer.iter().enumerate() {
                state.fixtures[index].programmer = stored.values;
            }
        }
        if let Some(recorded) = &document.recorded_slots {
            if recorded.len() != PAGE_COUNT || recorded.iter().any(|page| page.len() != SLOT_COUNT) {
                return Err("INVALID_SHOW: struktur recordedSlots tidak valid".to_string());
            }
            state.recorded = recorded.clone();
        }
        if let Some(levels) = &document.playback_levels {
            if levels.len() != PAGE_COUNT || levels.iter().any(|page| page.len() != SLOT_COUNT) {
                return Err("INVALID_SHOW: struktur playbackLevels tidak valid".to_string());
            }
            state.levels = levels.clone();
        }
        validate_shape_kind(&document.shape.kind)?;
        state.shape.kind = document.shape.kind.clone();
        state.shape.params = validate_shape_params(&document.shape.params)?;
        state.shape.playing = false;
        state.shape.phase_ms = 0;
        state.shape.targets = (0..FIXTURE_COUNT).collect();
        state.master = document.master;
        state.playing = document
            .playback_levels
            .as_ref()
            .is_some_and(|pages| pages.iter().flatten().any(|level| *level > 0));
        state.blackout = false;
        state.latch = None;
        state.frame_id = 0;
        Ok(state)
    }
}

fn require_slot(slot: Option<u8>, action: &str) -> Result<usize, String> {
    let slot = slot.ok_or_else(|| format!("INVALID_SLOT: aksi {action} memerlukan slot"))?;
    validate_slot(slot)
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

    pub fn shape_phase_ms(&self) -> u128 {
        match self.state.read() {
            Ok(guard) => guard.shape.phase_ms,
            Err(error) => {
                eprintln!("[kinglight] error=engine_state_poisoned detail={error}");
                0
            }
        }
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
                    master: u16::MAX,
                    shape_params: ShapeParams::default(),
                    playback_levels: [0; SLOT_COUNT],
                    recorded_slots: [false; SLOT_COUNT],
                    programmer_active: false,
                    channel_values: [0; 32],
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
        state.shape.targets = (0..FIXTURE_COUNT).collect();
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
        state.shape.targets = (0..FIXTURE_COUNT).collect();
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
        assert_ne!(
            shape_values("Circle", &state.shape.params, 0, 0),
            shape_values("Circle", &state.shape.params, 1_000, 0)
        );
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
    fn shape_kind_is_validated_against_eleven_names() {
        assert!(validate_shape_kind("Diamond").is_ok());
        assert!(validate_shape_kind("Dimmer").is_ok());
        assert!(validate_shape_kind("RGB").is_ok());
        assert!(validate_shape_kind("Color").is_ok());
        assert!(validate_shape_kind("Triangle").is_err());
    }
}
