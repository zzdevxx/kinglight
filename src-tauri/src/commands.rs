use std::sync::Arc;

use tauri::ipc::Channel;
use tauri::State;

use crate::engine::{EngineState, FrameBundle, validate_shape_kind, validate_shape_params};
use crate::model::{EncoderMode, FrameSnapshot, ShapeParams};
use crate::storage;
use crate::{FIXTURE_COUNT, Runtime};

async fn mutate<F>(state: &State<'_, Arc<Runtime>>, change: F) -> Result<FrameSnapshot, String>
where
    F: FnOnce(&mut EngineState) -> Result<(), String>,
{
    let output_state = state.status().await;
    state.engine().mutate(&output_state, change)
}

#[tauri::command]
pub async fn subscribe_frames(
    state: State<'_, Arc<Runtime>>,
    on_frame: Channel<FrameSnapshot>,
) -> Result<(), String> {
    state.subscribe(on_frame).await;
    Ok(())
}

#[tauri::command]
pub async fn get_snapshot(state: State<'_, Arc<Runtime>>) -> Result<FrameSnapshot, String> {
    Ok(state.snapshot().await)
}

#[tauri::command]
pub async fn select_fixture(
    state: State<'_, Arc<Runtime>>,
    slot: u8,
) -> Result<FrameSnapshot, String> {
    if slot == 0 || slot as usize > FIXTURE_COUNT {
        return Err("INVALID_FIXTURE_SLOT: slot harus 1 sampai 16".to_string());
    }
    mutate(&state, move |engine| {
        let index = slot as usize - 1;
        engine.fixtures[index].selected = !engine.fixtures[index].selected;
        Ok(())
    })
    .await
}

#[tauri::command]
pub async fn set_group(
    state: State<'_, Arc<Runtime>>,
    group: u8,
) -> Result<FrameSnapshot, String> {
    if group > 4 {
        return Err("INVALID_GROUP: group harus 0 sampai 4".to_string());
    }
    mutate(&state, move |engine| {
        engine.active_group = group;
        for fixture in &mut engine.fixtures {
            fixture.selected = false;
        }
        if group > 0 {
            let start = (group as usize - 1) * 4;
            for fixture in &mut engine.fixtures[start..start + 4] {
                fixture.selected = true;
            }
        }
        Ok(())
    })
    .await
}

#[tauri::command]
pub async fn set_page(state: State<'_, Arc<Runtime>>, page: u8) -> Result<FrameSnapshot, String> {
    if page == 0 || page > 6 {
        return Err("INVALID_PAGE: page harus 1 sampai 6".to_string());
    }
    mutate(&state, move |engine| {
        engine.active_page = page;
        let revision = engine.revisions.iter().copied().max().unwrap_or(0) + 1;
        engine.revisions[page as usize - 1] = revision;
        Ok(())
    })
    .await
}

#[tauri::command]
pub async fn set_bank(state: State<'_, Arc<Runtime>>, bank: u8) -> Result<FrameSnapshot, String> {
    if bank == 0 || bank > 2 {
        return Err("INVALID_BANK: bank harus 1 atau 2".to_string());
    }
    mutate(&state, move |engine| {
        engine.channel_bank = bank;
        Ok(())
    })
    .await
}

#[tauri::command]
pub async fn set_fader(
    state: State<'_, Arc<Runtime>>,
    attribute: String,
    value: u16,
) -> Result<FrameSnapshot, String> {
    let normalized = attribute.to_ascii_lowercase().replace(['_', '-'], "");
    mutate(&state, move |engine| {
        let indices = engine.target_indices();
        match normalized.as_str() {
            "master" => engine.master = value,
            "pancoarse" => engine.set_fine_byte(&indices, crate::engine::Attribute::Pan, true, value),
            "panfine" => engine.set_fine_byte(&indices, crate::engine::Attribute::Pan, false, value),
            "tiltcoarse" => engine.set_fine_byte(&indices, crate::engine::Attribute::Tilt, true, value),
            "tiltfine" => engine.set_fine_byte(&indices, crate::engine::Attribute::Tilt, false, value),
            name => {
                let parsed = crate::engine::Attribute::parse(name)
                    .ok_or_else(|| format!("INVALID_ATTRIBUTE: atribut {attribute} tidak dikenal"))?;
                engine.set_programmer(&indices, parsed, value);
            }
        }
        Ok(())
    })
    .await
}

#[tauri::command]
pub async fn set_encoder(
    state: State<'_, Arc<Runtime>>,
    axis: String,
    delta: i32,
    mode: EncoderMode,
) -> Result<FrameSnapshot, String> {
    let normalized = axis.to_ascii_lowercase();
    if normalized != "a" && normalized != "b" {
        return Err("INVALID_ENCODER_AXIS: axis harus a atau b".to_string());
    }
    mutate(&state, move |engine| {
        engine.encoder_mode = mode;
        let step = match mode {
            EncoderMode::Normal => 2048i64,
            EncoderMode::Fine => 256i64,
            EncoderMode::Ultra => 16i64,
        };
        let attribute = if normalized == "a" {
            crate::engine::Attribute::Pan
        } else {
            crate::engine::Attribute::Tilt
        };
        let indices = engine.target_indices();
        for index in indices {
            let current = engine.fixtures[index].programmer[attribute.index()]
                .unwrap_or(engine.fixtures[index].values[attribute.index()]);
            let next = (current as i64 + delta as i64 * step).clamp(0, u16::MAX as i64) as u16;
            engine.fixtures[index].programmer[attribute.index()] = Some(next);
        }
        Ok(())
    })
    .await
}

#[tauri::command]
pub async fn set_shape(
    state: State<'_, Arc<Runtime>>,
    kind: String,
    params: ShapeParams,
) -> Result<FrameSnapshot, String> {
    validate_shape_kind(&kind)?;
    let params = validate_shape_params(&params)?;
    mutate(&state, move |engine| {
        engine.shape.kind = kind;
        engine.shape.params = params;
        engine.shape.phase_ms = 0;
        Ok(())
    })
    .await
}

#[tauri::command]
pub async fn shape_play(
    state: State<'_, Arc<Runtime>>,
    active: bool,
) -> Result<FrameSnapshot, String> {
    mutate(&state, move |engine| {
        if active && !engine.shape.playing {
            engine.shape.phase_ms = 0;
        }
        engine.shape.playing = active;
        Ok(())
    })
    .await
}

#[tauri::command]
pub async fn set_blackout(
    state: State<'_, Arc<Runtime>>,
    active: bool,
) -> Result<FrameSnapshot, String> {
    mutate(&state, move |engine| {
        if active && !engine.blackout {
            let values = engine.resolved_values(false);
            engine.latch = Some(crate::engine::BlackoutLatch {
                pan: values.iter().map(|value| value.pan).collect(),
                tilt: values.iter().map(|value| value.tilt).collect(),
                color: values.iter().map(|value| value.color).collect(),
            });
        }
        if !active {
            engine.latch = None;
        }
        engine.blackout = active;
        Ok(())
    })
    .await
}

#[tauri::command]
pub async fn set_playing(
    state: State<'_, Arc<Runtime>>,
    active: bool,
) -> Result<FrameSnapshot, String> {
    mutate(&state, move |engine| {
        engine.playing = active;
        Ok(())
    })
    .await
}

#[tauri::command]
pub async fn configure_output(
    state: State<'_, Arc<Runtime>>,
    target: String,
    port_address: u32,
    compat_zero: bool,
) -> Result<FrameSnapshot, String> {
    let validated = crate::artnet::validate_port_address(port_address, compat_zero)?;
    state
        .output()
        .configure(target, validated, compat_zero)
        .await?;
    Ok(state.snapshot().await)
}

#[tauri::command]
pub async fn arm_output(state: State<'_, Arc<Runtime>>) -> Result<FrameSnapshot, String> {
    state.output().arm().await?;
    Ok(state.snapshot().await)
}

#[tauri::command]
pub async fn stop_output(state: State<'_, Arc<Runtime>>) -> Result<FrameSnapshot, String> {
    let frame: FrameBundle = state.engine().stop_frame();
    state.output().stop(frame).await?;
    Ok(state.snapshot().await)
}

#[tauri::command]
pub async fn save_show(
    state: State<'_, Arc<Runtime>>,
    name: String,
) -> Result<String, String> {
    let document = state.engine().document();
    let path = storage::save_document(state.storage_root(), &name, &document)?;
    Ok(path.to_string_lossy().into_owned())
}

#[tauri::command]
pub async fn load_show(
    state: State<'_, Arc<Runtime>>,
    name: String,
) -> Result<FrameSnapshot, String> {
    let output_state = state.status().await;
    if output_state.armed {
        return Err("OUTPUT_ARMED: stop output sebelum memuat show".to_string());
    }
    let document = storage::load_document(state.storage_root(), &name)?;
    state.engine().replace_show(&document)?;
    Ok(state.snapshot().await)
}

#[tauri::command]
pub async fn list_shows(state: State<'_, Arc<Runtime>>) -> Result<Vec<String>, String> {
    storage::list_documents(state.storage_root())
}
