pub mod artnet;
pub mod commands;
pub mod engine;
pub mod model;
pub mod storage;

use std::path::PathBuf;
use std::sync::{Arc, RwLock as StdRwLock};
use std::time::Duration;

use engine::Engine;
use model::{FrameSnapshot, OutputStatusCore};
use tauri::ipc::Channel;
use tauri::{Manager, WindowEvent};
use tokio::sync::RwLock;

pub const FIXTURE_COUNT: usize = 16;
pub const FIXTURE_FOOTPRINT: usize = 8;
pub const DMX_LEN: usize = 512;
pub const ART_DMX_LEN: usize = 530;
pub const ARTNET_PORT: u16 = 6454;
pub const ENGINE_TICK_MS: u64 = 20;
pub const PUBLISH_INTERVAL: Duration = Duration::from_micros(33_333);
pub const OUTPUT_INTERVAL: Duration = Duration::from_micros(33_333);

pub struct Runtime {
    engine: Arc<Engine>,
    output: Arc<artnet::OutputController>,
    status: Arc<RwLock<OutputStatusCore>>,
    channels: Arc<StdRwLock<Vec<Channel<FrameSnapshot>>>>,
    storage_root: PathBuf,
}

impl Runtime {
    pub fn new(storage_root: PathBuf) -> Self {
        let status = Arc::new(RwLock::new(OutputStatusCore::default()));
        let engine = Arc::new(Engine::new());
        let frames = Arc::new(RwLock::new(engine.frame()));
        let output = Arc::new(artnet::OutputController::start(frames.clone(), status.clone()));
        Self {
            engine,
            output,
            status,
            channels: Arc::new(StdRwLock::new(Vec::new())),
            storage_root,
        }
    }

    pub fn engine(&self) -> &Arc<Engine> {
        &self.engine
    }

    pub fn output(&self) -> &Arc<artnet::OutputController> {
        &self.output
    }

    pub fn storage_root(&self) -> &PathBuf {
        &self.storage_root
    }

    pub async fn status(&self) -> OutputStatusCore {
        self.status.read().await.clone()
    }

    pub async fn snapshot(&self) -> FrameSnapshot {
        let status = self.status.read().await.clone();
        self.engine.snapshot(&status)
    }

    pub fn spawn_engine_tick(self: &Arc<Self>) {
        let runtime = Arc::clone(self);
        tauri::async_runtime::spawn(async move {
            let mut next = tokio::time::Instant::now();
            let mut last_publish = next - Duration::from_millis(200);
            loop {
                tokio::time::sleep_until(next).await;
                let started = tokio::time::Instant::now();
                let lag = started.saturating_duration_since(next);
                runtime
                    .engine
                    .advance(ENGINE_TICK_MS as u128 + lag.as_millis() as u128);
                next += Duration::from_millis(ENGINE_TICK_MS);
                if next < tokio::time::Instant::now() {
                    next = tokio::time::Instant::now() + Duration::from_millis(ENGINE_TICK_MS);
                }
                if last_publish.elapsed() >= PUBLISH_INTERVAL {
                    runtime.publish().await;
                    last_publish = tokio::time::Instant::now();
                }
            }
        });
    }

    pub async fn subscribe(&self, channel: Channel<FrameSnapshot>) {
        {
            let mut channels = match self.channels.write() {
                Ok(guard) => guard,
                Err(error) => {
                    eprintln!("[kinglight] error=frame_channels_poisoned detail={error}");
                    return;
                }
            };
            channels.push(channel.clone());
        }
        if channel.send(self.snapshot().await).is_err() {
            self.remove_channels(&[channel.id()]);
        }
    }

    async fn publish(&self) {
        let snapshot = self.snapshot().await;
        let channels = match self.channels.read() {
            Ok(guard) => guard.clone(),
            Err(error) => {
                eprintln!("[kinglight] error=frame_channels_poisoned detail={error}");
                return;
            }
        };
        let mut dead = Vec::new();
        for channel in &channels {
            if channel.send(snapshot.clone()).is_err() {
                dead.push(channel.id());
            }
        }
        if !dead.is_empty() {
            self.remove_channels(&dead);
        }
    }

    fn remove_channels(&self, ids: &[u32]) {
        let mut channels = match self.channels.write() {
            Ok(guard) => guard,
            Err(error) => {
                eprintln!("[kinglight] error=frame_channels_poisoned detail={error}");
                return;
            }
        };
        channels.retain(|channel| !ids.contains(&channel.id()));
    }

    pub fn shutdown(&self) {
        self.output.shutdown();
    }
}

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let data_dir = app
                .path()
                .app_data_dir()
                .map_err(|error| format!("APP_DATA_DIR: {error}"))?;
            storage::ensure_data_dir(&data_dir)?;
            let runtime = Arc::new(Runtime::new(data_dir));
            runtime.spawn_engine_tick();
            app.manage(runtime);
            Ok(())
        })
        .on_window_event(|window, event| {
            if matches!(event, WindowEvent::Destroyed) {
                if let Some(runtime) = window.app_handle().try_state::<Arc<Runtime>>() {
                    runtime.shutdown();
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::subscribe_frames,
            commands::get_snapshot,
            commands::select_fixture,
            commands::set_group,
            commands::set_page,
            commands::set_bank,
            commands::set_fader,
            commands::set_encoder,
            commands::set_shape,
            commands::shape_play,
            commands::set_blackout,
            commands::set_playing,
            commands::console_action,
            commands::set_playback,
            commands::configure_output,
            commands::arm_output,
            commands::stop_output,
            commands::save_show,
            commands::load_show,
            commands::list_shows,
        ])
        .run(tauri::generate_context!())
        .expect("Kinglight gagal memulai aplikasi");
}
