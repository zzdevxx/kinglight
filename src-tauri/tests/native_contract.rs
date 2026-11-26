use std::collections::HashSet;

use kinglight_lib::engine::{
    shape_values_for, validate_shape_params, Engine, EngineState, SHAPE_KINDS, validate_shape_kind,
};
use kinglight_lib::model::{
    EncoderMode, Interleave, OutputStatusCore, ShapeDirection, ShapeParams,
};
use kinglight_lib::storage::{validate_document, ShowDocument};

fn output() -> OutputStatusCore {
    OutputStatusCore::default()
}

fn act<F>(engine: &Engine, change: F)
where
    F: FnOnce(&mut EngineState) -> Result<(), String>,
{
    engine.mutate(&output(), change).expect("mutate should succeed");
}

fn params(speed: f64) -> ShapeParams {
    ShapeParams {
        speed,
        bpm: None,
        size: 0.4,
        spread: 0.04,
        repeat: 1,
        interleave: Interleave::None,
        direction: ShapeDirection::Left,
        amplitude: 0.4,
    }
}

fn edit_two_fixture_cue(engine: &Engine) {
    act(&engine, |state| state.toggle_fixture(1));
    act(&engine, |state| state.set_fader_attribute("ch1", 40_000));
    act(&engine, |state| state.set_fader_attribute("ch2", 12_000));
    act(&engine, |state| state.set_fader_attribute("ch6", 50_000));
    act(&engine, |state| state.toggle_fixture(1));

    act(&engine, |state| state.toggle_fixture(2));
    act(&engine, |state| state.set_fader_attribute("ch1", 20_000));
    act(&engine, |state| state.set_fader_attribute("ch2", 50_000));
    act(&engine, |state| state.set_fader_attribute("ch6", 10_000));
    act(&engine, |state| state.toggle_fixture(2));
}

#[test]
fn startup_has_fan_first_twelve_dark_tail_and_no_selection() {
    let frame = Engine::new().snapshot(&output());
    assert_eq!(frame.fixtures.len(), 16);
    assert_eq!(frame.master, u16::MAX);
    assert_eq!(frame.shape_params, ShapeParams::default());
    assert_eq!(frame.shape_params.bpm, Some(6.0));
    assert!((frame.shape_params.size - 25.0 / 255.0).abs() < 1e-9);
    assert_eq!(frame.shape_params.spread, 1.0);
    assert_eq!(frame.shape_params.repeat, 8);
    assert_eq!(frame.shape_params.amplitude, 0.5);
    assert_eq!(frame.playback_levels, [0u16; 16]);
    assert_eq!(frame.recorded_slots, [false; 16]);
    assert!(!frame.programmer_active);
    assert!(!frame.output.armed);
    assert!(!frame.shape_playing);
    assert_eq!(frame.channel_values.len(), 32);

    let pans: Vec<f64> = frame.fixtures[..12]
        .iter()
        .map(|fixture| fixture.pan)
        .collect();
    assert!(pans.windows(2).all(|pair| pair[0] < pair[1]));
    assert!((pans[0] - 0.08).abs() < 0.01);
    assert!((pans[11] - 0.92).abs() < 0.01);
    for fixture in &frame.fixtures[..12] {
        assert!(fixture.dimmer > 0.99);
        assert!((fixture.tilt - 0.5).abs() < 0.01);
        assert_eq!(fixture.color, [1.0, 1.0, 1.0]);
    }
    for fixture in &frame.fixtures[12..] {
        assert_eq!(fixture.dimmer, 0.0);
        assert!((fixture.pan - 0.5).abs() < 0.01);
    }
    assert!(frame.fixtures.iter().all(|fixture| !fixture.selected));
}

#[test]
fn console_action_rejects_unknown_actions_and_out_of_range_slots() {
    let mut state = EngineState::new();
    assert!(state.console_action("bogus", None, None).is_err());
    assert!(state.console_action("record", Some(0), None).is_err());
    assert!(state.console_action("record", Some(17), None).is_err());
    assert!(state.console_action("include", None, None).is_err());
    assert!(state
        .console_action("copy", Some(2), Some(0))
        .is_err());
    assert!(state
        .console_action("move", Some(2), Some(17))
        .is_err());
    assert!(state.console_action("copy", Some(2), None).is_err());
}

#[test]
fn console_actions_record_include_copy_move_delete_and_clear() {
    let engine = Engine::new();
    act(&engine, |state| state.toggle_fixture(1));
    act(&engine, |state| state.set_fader_attribute("ch1", 40_000));
    let edited = engine.snapshot(&output());
    assert!(edited.programmer_active);
    assert_eq!(edited.channel_values[0], (40_000 >> 8) as u8);

    act(&engine, |state| state.console_action("record", Some(3), None));
    let recorded = engine.snapshot(&output());
    assert!(recorded.recorded_slots[2]);

    act(&engine, |state| state.console_action("clear", None, None));
    let cleared = engine.snapshot(&output());
    assert!(!cleared.programmer_active);
    assert!(cleared.fixtures.iter().all(|fixture| !fixture.selected));
    assert!((cleared.fixtures[0].dimmer - 1.0).abs() < 0.01);

    act(&engine, |state| state.console_action("include", Some(3), None));
    let included = engine.snapshot(&output());
    assert!(included.programmer_active);
    assert!((included.fixtures[0].dimmer * 65_535.0 - 40_000.0).abs() < 2.0);

    act(&engine, |state| state.console_action("copy", Some(4), Some(3)));
    act(&engine, |state| state.console_action("move", Some(5), Some(4)));
    let copied = engine.snapshot(&output());
    assert!(copied.recorded_slots[2]);
    assert!(!copied.recorded_slots[3]);
    assert!(copied.recorded_slots[4]);

    act(&engine, |state| state.console_action("delete", Some(5), None));
    let deleted = engine.snapshot(&output());
    assert!(!deleted.recorded_slots[4]);
}

#[test]
fn record_and_include_restore_distinct_fixture_cues() {
    let engine = Engine::new();
    edit_two_fixture_cue(&engine);
    let before = engine.snapshot(&output());
    assert_ne!(before.fixtures[0].dimmer, before.fixtures[1].dimmer);
    assert_ne!(before.fixtures[0].pan, before.fixtures[1].pan);
    assert_ne!(before.fixtures[0].color[0], before.fixtures[1].color[0]);

    act(&engine, |state| state.console_action("record", Some(3), None));
    act(&engine, |state| state.console_action("clear", None, None));
    act(&engine, |state| state.console_action("include", Some(3), None));
    let restored = engine.snapshot(&output());

    assert_eq!(restored.dmx, before.dmx);
    assert_eq!(restored.channel_values, before.channel_values);
    assert_eq!(restored.fixtures[0].dimmer, before.fixtures[0].dimmer);
    assert_eq!(restored.fixtures[1].dimmer, before.fixtures[1].dimmer);
    assert_eq!(restored.fixtures[0].pan, before.fixtures[0].pan);
    assert_eq!(restored.fixtures[1].pan, before.fixtures[1].pan);
    assert_eq!(restored.fixtures[0].color[0], before.fixtures[0].color[0]);
    assert_eq!(restored.fixtures[1].color[0], before.fixtures[1].color[0]);
}

#[test]
fn playback_full_frame_preserves_distinct_fixture_cues() {
    let engine = Engine::new();
    edit_two_fixture_cue(&engine);
    let before = engine.snapshot(&output());
    act(&engine, |state| state.console_action("record", Some(3), None));
    act(&engine, |state| state.console_action("clear", None, None));
    act(&engine, |state| state.set_playback(3, u16::MAX));

    let active = engine.snapshot(&output());
    assert!(active.output.playing);
    assert_eq!(active.dmx, before.dmx);
    assert_eq!(active.channel_values, before.channel_values);
    assert_ne!(active.fixtures[0].dimmer, active.fixtures[1].dimmer);
    assert_ne!(active.fixtures[0].pan, active.fixtures[1].pan);
    assert_ne!(active.fixtures[0].color[0], active.fixtures[1].color[0]);
}

#[test]
fn showfile_cue_roundtrip_retains_per_fixture_values() {
    let engine = Engine::new();
    edit_two_fixture_cue(&engine);
    let before = engine.snapshot(&output());
    act(&engine, |state| state.console_action("record", Some(3), None));

    let document = engine.document();
    let cue = &document.pages[0][2];
    let fixtures = cue.fixtures.as_ref().expect("cue stores per-fixture values");
    assert_eq!(fixtures.len(), 16);
    assert_ne!(fixtures[0].values, fixtures[1].values);

    let mut json = serde_json::to_value(&document).unwrap();
    json.as_object_mut()
        .unwrap()
        .remove("programmer");
    let loaded: ShowDocument = serde_json::from_value(json).unwrap();
    let restored = Engine::new();
    restored.replace_show(&loaded).expect("show replaces state");
    act(&restored, |state| state.console_action("include", Some(3), None));
    let after = restored.snapshot(&output());
    assert_eq!(after.dmx, before.dmx);
    assert_eq!(after.channel_values, before.channel_values);
    assert_ne!(after.fixtures[0].pan, after.fixtures[1].pan);
    assert_ne!(after.fixtures[0].color[0], after.fixtures[1].color[0]);
}

#[test]
fn select_all_locate_and_group_resets_are_consistent() {
    let engine = Engine::new();
    act(&engine, |state| state.set_group(2));
    assert_eq!(engine.snapshot(&output()).active_group, 2);
    act(&engine, |state| state.toggle_fixture(1));
    let toggled = engine.snapshot(&output());
    assert_eq!(toggled.active_group, 0);
    assert!(toggled.fixtures[0].selected);
    assert!(toggled.fixtures[5..9].iter().all(|fixture| !fixture.selected));

    act(&engine, |state| state.console_action("select_all", None, None));
    assert!(engine
        .snapshot(&output())
        .fixtures
        .iter()
        .all(|fixture| fixture.selected));

    act(&engine, |state| state.console_action("clear", None, None));
    act(&engine, |state| state.console_action("locate", None, None));
    let located = engine.snapshot(&output());
    assert!(located.programmer_active);
    assert!(located.fixtures.iter().all(|fixture| fixture.dimmer > 0.99));
}

#[test]
fn playback_fader_activates_recorded_scene_and_reports_level() {
    let engine = Engine::new();
    act(&engine, |state| state.toggle_fixture(1));
    act(&engine, |state| state.set_fader_attribute("ch1", 40_000));
    act(&engine, |state| state.console_action("record", Some(3), None));
    act(&engine, |state| state.console_action("clear", None, None));

    act(&engine, |state| state.set_playback(3, 32_768));
    let active = engine.snapshot(&output());
    assert_eq!(active.playback_levels[2], 32_768);
    assert!(active.output.playing);
    assert!((active.fixtures[0].dimmer * 65_535.0 - 20_000.0).abs() < 100.0);

    assert!(engine
        .mutate(&output(), |state| state.set_playback(0, 1_000))
        .is_err());
    assert!(engine
        .mutate(&output(), |state| state.set_playback(17, 1_000))
        .is_err());

    act(&engine, |state| Ok(state.set_playing(false)));
    let stopped = engine.snapshot(&output());
    assert!((stopped.fixtures[0].dimmer - 1.0).abs() < 0.01);
}

#[test]
fn channel_faders_follow_selected_fixture_footprint() {
    let engine = Engine::new();
    act(&engine, |state| state.toggle_fixture(3));
    act(&engine, |state| state.set_fader_attribute("ch1", 65_535));
    act(&engine, |state| state.set_fader_attribute("ch2", 65_535));
    let frame = engine.snapshot(&output());
    assert!((frame.fixtures[2].dimmer - 1.0).abs() < 0.01);
    assert!(frame.fixtures[2].pan > 0.99);
    assert_eq!(frame.channel_values[0], u8::MAX);
    assert_eq!(frame.channel_values[1], u8::MAX);

    for attribute in ["ch9", "ch16", "ch33"] {
        assert!(engine
            .mutate(&output(), |state| state.set_fader_attribute(attribute, 1_000))
            .is_err());
    }
    assert!(engine
        .mutate(&output(), |state| state.set_fader_attribute("ch1", 1_000))
        .is_ok());
}

#[test]
fn encoder_delta_is_one_tick_and_scaled_once_by_mode() {
    let engine = Engine::new();
    act(&engine, |state| state.toggle_fixture(1));
    let before = engine.snapshot(&output()).fixtures[0].pan;

    act(&engine, |state| {
        state.encoder_tick("a", 1, EncoderMode::Normal)
    });
    let normal = engine.snapshot(&output()).fixtures[0].pan;
    act(&engine, |state| state.encoder_tick("a", 1, EncoderMode::Fine));
    let fine = engine.snapshot(&output()).fixtures[0].pan;
    act(&engine, |state| state.encoder_tick("a", -1, EncoderMode::Ultra));
    let ultra = engine.snapshot(&output()).fixtures[0].pan;

    assert!((normal - before - 1024.0 / 65_535.0).abs() < 1e-6);
    assert!((fine - normal - 128.0 / 65_535.0).abs() < 1e-6);
    assert!((ultra - fine + 16.0 / 65_535.0).abs() < 1e-6);
}

#[test]
fn encoder_mode_only_does_not_touch_programmer_or_shape_motion() {
    let engine = Engine::new();
    act(&engine, |state| state.apply_shape("PanSwing", params(1.0)));
    act(&engine, |state| Ok(state.shape_play(true)));
    engine.advance(100);
    let before = engine.snapshot(&output());
    assert!(!before.programmer_active);

    act(&engine, |state| state.encoder_tick("a", 0, EncoderMode::Fine));
    let after_mode = engine.snapshot(&output());
    assert_eq!(after_mode.encoder_mode, EncoderMode::Fine);
    assert!(!after_mode.programmer_active);
    assert_eq!(after_mode.dmx[..128], before.dmx[..128]);

    engine.advance(100);
    let after_motion = engine.snapshot(&output());
    assert_ne!(after_motion.dmx[..128], after_mode.dmx[..128]);
}

#[test]
fn shape_kinds_include_three_new_categories_with_real_effects() {
    assert_eq!(SHAPE_KINDS.len(), 11);
    for kind in ["Dimmer", "RGB", "Color"] {
        assert!(validate_shape_kind(kind).is_ok());
    }
    assert!(validate_shape_kind("Triangle").is_err());

    let engine = Engine::new();
    let base = engine.snapshot(&output());
    act(&engine, |state| state.apply_shape("Dimmer", params(1.0)));
    act(&engine, |state| Ok(state.shape_play(true)));
    let dimmer = engine.snapshot(&output());
    assert!(dimmer.shape_playing);
    assert_ne!(dimmer.dmx[0], base.dmx[0]);
    assert!(dimmer.fixtures[0].dimmer < 1.0);

    act(&engine, |state| state.apply_shape("RGB", params(1.0)));
    let rgb = engine.snapshot(&output());
    assert_ne!(rgb.dmx[5], dimmer.dmx[5]);

    act(&engine, |state| state.apply_shape("Color", params(1.0)));
    let color = engine.snapshot(&output());
    assert_ne!(color.dmx[5], rgb.dmx[5]);
}

#[test]
fn shape_parameter_edits_keep_phase_and_kind_change_resets_it() {
    let engine = Engine::new();
    act(&engine, |state| state.apply_shape("Circle", params(1.0)));
    act(&engine, |state| Ok(state.shape_play(true)));
    engine.advance(100);
    let before_phase = engine.shape_phase_ms();
    let before = engine.snapshot(&output());

    act(&engine, |state| state.apply_shape("Circle", params(2.0)));
    let after_params = engine.snapshot(&output());
    assert_eq!(engine.shape_phase_ms(), before_phase);
    assert_eq!(after_params.shape_kind, "Circle");
    assert_eq!(after_params.shape_params.speed, 2.0);

    act(&engine, |state| state.apply_shape("Figure8", params(2.0)));
    let after_kind = engine.snapshot(&output());
    assert_eq!(engine.shape_phase_ms(), 0);
    assert_ne!(after_kind.dmx[..128], before.dmx[..128]);
}

#[test]
fn spread_distributes_phase_and_repeat_does_not_change_temporal_rate() {
    let mut wide = params(1.0);
    wide.spread = 1.0;
    wide.bpm = Some(6.0);
    let first = shape_values_for("PanSwing", &wide, 500, 0, 12);
    let second = shape_values_for("PanSwing", &wide, 500, 1, 12);
    assert_ne!(first, second);

    let mut repeated = wide.clone();
    repeated.repeat = 8;
    for phase in [0u128, 250, 500, 750, 1_000] {
        assert_eq!(
            shape_values_for("PanSwing", &wide, phase, 0, 12),
            shape_values_for("PanSwing", &repeated, phase, 0, 12)
        );
    }
    assert_ne!(
        shape_values_for("PanSwing", &wide, 500, 1, 12),
        shape_values_for("PanSwing", &repeated, 500, 1, 12)
    );

    let engine = Engine::new();
    let base = engine.snapshot(&output());
    act(&engine, |state| state.toggle_fixture(5));
    act(&engine, |state| state.toggle_fixture(9));
    act(&engine, |state| state.apply_shape("PanSwing", wide));
    act(&engine, |state| Ok(state.shape_play(true)));
    engine.advance(500);
    let frame = engine.snapshot(&output());
    assert_ne!(frame.fixtures[4].pan, frame.fixtures[8].pan);
    assert_eq!(frame.fixtures[0].pan, base.fixtures[0].pan);
    assert_eq!(frame.fixtures[1].pan, base.fixtures[1].pan);
}

#[test]
fn circle_two_way_distributes_180_degree_spread_without_changing_bpm() {
    let mut video = params(1.0);
    video.bpm = Some(6.0);
    video.size = 25.0 / 255.0;
    video.spread = 1.0;
    video.repeat = 8;
    video.interleave = Interleave::None;
    let validated = validate_shape_params(&video).expect("video shape is valid");
    assert_eq!(validated.bpm, Some(6.0));

    let positions: HashSet<(u16, u16)> = (0..16)
        .map(|index| shape_values_for("CircleTwoWay", &validated, 0, index, 16))
        .collect();
    assert!(positions.len() >= 6, "expected distributed fixtures");

    let mut no_repeat = validated.clone();
    no_repeat.repeat = 1;
    for phase in [0u128, 250, 500, 750] {
        assert_eq!(
            shape_values_for("CircleTwoWay", &validated, phase, 0, 16),
            shape_values_for("CircleTwoWay", &no_repeat, phase, 0, 16)
        );
    }
}

#[test]
fn repeat_zero_is_accepted_and_produces_finite_output() {
    let mut zero = params(1.0);
    zero.spread = 1.0;
    zero.repeat = 0;
    let validated = validate_shape_params(&zero).expect("repeat 0 is valid");
    assert_eq!(validated.repeat, 0);

    let first = shape_values_for("PanSwing", &validated, 500, 0, 12);
    let second = shape_values_for("PanSwing", &validated, 500, 1, 12);
    assert_eq!(first, second);
    let pan = f64::from(first.0) / f64::from(u16::MAX);
    let tilt = f64::from(first.1) / f64::from(u16::MAX);
    assert!(pan.is_finite() && (0.0..=1.0).contains(&pan));
    assert!(tilt.is_finite() && (0.0..=1.0).contains(&tilt));

    let engine = Engine::new();
    act(&engine, |state| state.apply_shape("PanSwing", zero));
    act(&engine, |state| Ok(state.shape_play(true)));
    engine.advance(250);
    let frame = engine.snapshot(&output());
    assert_eq!(frame.shape_params.repeat, 0);
    assert_eq!(frame.dmx.len(), 512);
    assert!(!frame.programmer_active);
}

#[test]
fn shape_targets_are_captured_from_selection_at_start() {
    let engine = Engine::new();
    act(&engine, |state| state.toggle_fixture(1));
    act(&engine, |state| state.apply_shape("PanSwing", params(1.0)));
    act(&engine, |state| Ok(state.shape_play(true)));
    let before = engine.snapshot(&output());
    act(&engine, |state| state.toggle_fixture(2));
    engine.advance(250);
    let after = engine.snapshot(&output());

    assert_ne!(after.fixtures[0].pan, before.fixtures[0].pan);
    assert_eq!(after.fixtures[1].pan, before.fixtures[1].pan);
}

#[test]
fn showfile_roundtrip_and_old_schema_defaults_preserve_programmer_and_scenes() {
    let engine = Engine::new();
    act(&engine, |state| state.toggle_fixture(1));
    act(&engine, |state| state.set_fader_attribute("ch1", 50_000));
    act(&engine, |state| state.console_action("record", Some(6), None));
    act(&engine, |state| state.set_playback(6, 20_000));

    let document = engine.document();
    assert_eq!(document.schema_version, 1);
    assert!(validate_document(&document).is_ok());
    let saved = document
        .recorded_slots
        .as_ref()
        .expect("recorded slots are saved");
    assert!(saved[0][5]);
    assert_eq!(
        document.playback_levels.as_ref().unwrap()[0][5],
        20_000
    );
    assert!(document.programmer.as_ref().unwrap()[0]
        .values
        .iter()
        .any(Option::is_some));

    let mut json = serde_json::to_value(&document).unwrap();
    json.as_object_mut()
        .unwrap()
        .remove("programmer");
    json.as_object_mut()
        .unwrap()
        .remove("recordedSlots");
    json.as_object_mut()
        .unwrap()
        .remove("playbackLevels");
    for page in json
        .as_object_mut()
        .unwrap()
        .get_mut("pages")
        .unwrap()
        .as_array_mut()
        .unwrap()
    {
        for slot in page.as_array_mut().unwrap() {
            slot.as_object_mut().unwrap().remove("fixtures");
        }
    }
    let old: ShowDocument = serde_json::from_value(json).unwrap();
    assert!(validate_document(&old).is_ok());
    assert!(old.programmer.is_none());
    assert!(old.recorded_slots.is_none());
    assert!(old.playback_levels.is_none());
    assert!(old.pages.iter().flatten().all(|slot| slot.fixtures.is_none()));

    let restored = EngineState::replace_from_document(&old).expect("old show loads");
    let restored_engine = Engine::new();
    restored_engine
        .replace_show(&old)
        .expect("old show replaces state");
    let frame = restored_engine.snapshot(&output());
    assert!(!frame.programmer_active);
    assert_eq!(frame.playback_levels, [0u16; 16]);
    assert_eq!(frame.recorded_slots, [false; 16]);
    drop(restored);
}
