use std::sync::Arc;

use tauri::ipc::Channel;
use tauri::State;

use crate::engine::{EngineState, FrameBundle};
use crate::model::{EncoderMode, FrameSnapshot, ShapeParams};
use crate::storage;
use crate::Runtime;

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
    mutate(&state, move |engine| engine.toggle_fixture(slot)).await
}

#[tauri::command]
pub async fn set_group(state: State<'_, Arc<Runtime>>, group: u8) -> Result<FrameSnapshot, String> {
    mutate(&state, move |engine| engine.set_group(group)).await
}

#[tauri::command]
pub async fn set_page(state: State<'_, Arc<Runtime>>, page: u8) -> Result<FrameSnapshot, String> {
    mutate(&state, move |engine| engine.set_page(page)).await
}

#[tauri::command]
pub async fn set_bank(state: State<'_, Arc<Runtime>>, bank: u8) -> Result<FrameSnapshot, String> {
    mutate(&state, move |engine| engine.set_bank(bank)).await
}

#[tauri::command]
pub async fn set_fader(
    state: State<'_, Arc<Runtime>>,
    attribute: String,
    value: u16,
) -> Result<FrameSnapshot, String> {
    mutate(&state, move |engine| {
        engine.set_fader_attribute(&attribute, value)
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
    mutate(&state, move |engine| engine.encoder_tick(&axis, delta, mode)).await
}

#[tauri::command]
pub async fn set_shape(
    state: State<'_, Arc<Runtime>>,
    kind: String,
    params: ShapeParams,
) -> Result<FrameSnapshot, String> {
    mutate(&state, move |engine| engine.apply_shape(&kind, params)).await
}

#[tauri::command]
pub async fn shape_play(
    state: State<'_, Arc<Runtime>>,
    active: bool,
) -> Result<FrameSnapshot, String> {
    mutate(&state, move |engine| {
        engine.shape_play(active);
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
        engine.set_blackout(active);
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
        engine.set_playing(active);
        Ok(())
    })
    .await
}

#[tauri::command]
pub async fn console_action(
    state: State<'_, Arc<Runtime>>,
    action: String,
    slot: Option<u8>,
    source: Option<u8>,
) -> Result<FrameSnapshot, String> {
    mutate(&state, move |engine| engine.console_action(&action, slot, source)).await
}

#[tauri::command]
pub async fn set_playback(
    state: State<'_, Arc<Runtime>>,
    slot: u8,
    value: u16,
) -> Result<FrameSnapshot, String> {
    mutate(&state, move |engine| engine.set_playback(slot, value)).await
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
