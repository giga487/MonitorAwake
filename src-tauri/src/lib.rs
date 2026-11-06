pub mod core;
pub mod http;
pub mod platform;
pub mod service;

use core::Timer;
use fs2::FileExt;
use service::{Service, SharedService};
use std::{
    fs::{self, File, OpenOptions},
    path::PathBuf,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::{Duration, Instant},
};
use tokio::sync::watch;

pub struct AppRuntime {
    pub service: SharedService,
    pub config_url: String,
    shutdown: watch::Sender<bool>,
    stopped: Arc<AtomicBool>,
    worker: Mutex<Option<thread::JoinHandle<()>>>,
    _lock: File,
}

pub fn data_directory() -> Result<PathBuf, String> {
    if let Some(path) = std::env::var_os("MONITORAWAKE_DATA_DIR") {
        return Ok(PathBuf::from(path));
    }
    dirs::data_local_dir()
        .map(|p| p.join("MonitorAwake"))
        .ok_or("Cartella dati utente non disponibile.".into())
}

pub type Notify = Box<dyn Fn(&Timer) -> Result<(), String> + Send + 'static>;

pub async fn start(directory: PathBuf, notify: Notify) -> Result<AppRuntime, String> {
    fs::create_dir_all(&directory).map_err(|e| format!("Cartella dati: {e}"))?;
    let lock = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(directory.join("instance.lock"))
        .map_err(|e| e.to_string())?;
    lock.try_lock_exclusive()
        .map_err(|_| "MonitorAwake è già in esecuzione con questa cartella dati.".to_string())?;
    let service = Arc::new(Mutex::new(Service::load(directory.join("state.json"))?));
    let listener = tokio::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))
        .await
        .map_err(|e| format!("Server locale: {e}"))?;
    let address = listener.local_addr().map_err(|e| e.to_string())?;
    let token = uuid::Uuid::new_v4().simple().to_string();
    let config_url = format!("http://{address}/#token={token}");
    let router = http::router(http::HttpState {
        service: service.clone(),
        token: Arc::from(token),
        authority: Arc::from(address.to_string()),
    });
    let (shutdown, mut receiver) = watch::channel(false);
    let server_service = service.clone();
    tokio::spawn(async move {
        if let Err(error) = axum::serve(listener, router)
            .with_graceful_shutdown(async move {
                let _ = receiver.changed().await;
            })
            .await
            && let Ok(mut service) = server_service.lock()
        {
            service.backend_error = Some(format!("Server HTTP interrotto: {error}"));
        }
    });
    let stopped = Arc::new(AtomicBool::new(false));
    let worker_stop = stopped.clone();
    let worker_service = service.clone();
    let worker = thread::Builder::new()
        .name("monitorawake-scheduler".into())
        .spawn(move || {
            let mut native = platform::NativeAwake::default();
            if let Ok(mut service) = worker_service.lock() {
                service.capabilities = native.capabilities();
            }
            let mut last_attempt = Instant::now() - Duration::from_secs(10);
            let mut last_requested = core::AwakeMode::Off;
            while !worker_stop.load(Ordering::Acquire) {
                let (expired, notifications, requested) = match worker_service.lock() {
                    Ok(mut service) => service.tick(),
                    Err(_) => break,
                };
                if let Ok(mut service) = worker_service.lock()
                    && requested != service.applied_mode
                    && (requested != last_requested
                        || last_attempt.elapsed() >= Duration::from_secs(5))
                {
                    last_attempt = Instant::now();
                    last_requested = requested;
                    match native.set_mode(requested) {
                        Ok(()) => {
                            service.applied_mode = requested;
                            service.backend_error = None;
                        }
                        Err(error) => service.backend_error = Some(error),
                    }
                }
                if notifications {
                    for timer in &expired {
                        if let Err(error) = notify(timer)
                            && let Ok(mut service) = worker_service.lock()
                        {
                            service.notification_error = Some(error);
                        }
                    }
                }
                thread::sleep(Duration::from_millis(250));
            }
            // Dropping on this same thread releases the native awake request.
            drop(native);
        })
        .map_err(|e| e.to_string())?;
    Ok(AppRuntime {
        service,
        config_url,
        shutdown,
        stopped,
        worker: Mutex::new(Some(worker)),
        _lock: lock,
    })
}

impl AppRuntime {
    pub fn stop(&self) {
        self.stopped.store(true, Ordering::Release);
        let _ = self.shutdown.send(true);
        if let Ok(mut handle) = self.worker.lock()
            && let Some(handle) = handle.take()
        {
            let _ = handle.join();
        }
    }
}

impl Drop for AppRuntime {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(feature = "desktop")]
mod desktop {
    use super::*;
    use tauri::{Manager, State};
    use tauri_plugin_notification::NotificationExt;
    use tauri_plugin_opener::OpenerExt;

    #[tauri::command]
    fn get_state(runtime: State<'_, AppRuntime>) -> Result<service::Snapshot, String> {
        Ok(runtime
            .service
            .lock()
            .map_err(|_| "Motore non disponibile")?
            .snapshot())
    }

    #[tauri::command]
    fn dispatch(
        action: core::Action,
        runtime: State<'_, AppRuntime>,
    ) -> Result<service::ActionResult, String> {
        runtime
            .service
            .lock()
            .map_err(|_| "Motore non disponibile")?
            .dispatch(action)
    }

    #[tauri::command]
    fn configuration_url(runtime: State<'_, AppRuntime>) -> String {
        runtime.config_url.clone()
    }

    #[tauri::command]
    fn open_configuration(
        app: tauri::AppHandle,
        runtime: State<'_, AppRuntime>,
    ) -> Result<(), String> {
        // Called only by the user's Configurazione button. No browser opens at startup.
        app.opener()
            .open_url(&runtime.config_url, None::<&str>)
            .map_err(|e| e.to_string())
    }

    #[tauri::command]
    fn quit(app: tauri::AppHandle) {
        app.exit(0);
    }

    pub fn run() -> Result<(), String> {
        let app = tauri::Builder::default()
            .plugin(tauri_plugin_notification::init())
            .plugin(tauri_plugin_opener::init())
            .invoke_handler(tauri::generate_handler![
                get_state,
                dispatch,
                configuration_url,
                open_configuration,
                quit
            ])
            .setup(|app| {
                let handle = app.handle().clone();
                let directory = data_directory().map_err(std::io::Error::other)?;
                let runtime = tauri::async_runtime::block_on(start(
                    directory,
                    Box::new(move |timer| {
                        handle
                            .notification()
                            .builder()
                            .title("Tempo scaduto")
                            .body(format!("Il timer «{}» è terminato.", timer.name))
                            .show()
                            .map_err(|e| e.to_string())
                    }),
                ))
                .map_err(std::io::Error::other)?;
                app.manage(runtime);
                Ok(())
            })
            .build(tauri::generate_context!())
            .map_err(|e| e.to_string())?;
        app.run(|app, event| {
            if matches!(event, tauri::RunEvent::Exit)
                && let Some(runtime) = app.try_state::<AppRuntime>()
            {
                runtime.stop();
            }
        });
        Ok(())
    }
}

#[cfg(feature = "desktop")]
pub use desktop::run;
