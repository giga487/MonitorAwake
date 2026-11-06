use crate::core::AwakeMode;
use serde::Serialize;

#[derive(Clone, Serialize)]
pub struct Capabilities {
    pub system: bool,
    pub display: bool,
    pub backend: String,
}

// This value is created and used exclusively on the scheduler thread.
// Windows execution-state requests belong to that thread, not the async runtime.
#[cfg_attr(not(target_os = "linux"), derive(Default))]
pub struct NativeAwake {
    #[cfg(target_os = "linux")]
    connection: Option<zbus::blocking::Connection>,
    #[cfg(target_os = "linux")]
    inhibitor: Option<zbus::zvariant::OwnedFd>,
}

#[cfg(target_os = "linux")]
impl Default for NativeAwake {
    fn default() -> Self {
        Self {
            #[cfg(target_os = "linux")]
            connection: zbus::blocking::Connection::system().ok(),
            #[cfg(target_os = "linux")]
            inhibitor: None,
        }
    }
}

impl NativeAwake {
    pub fn capabilities(&self) -> Capabilities {
        Capabilities {
            system: cfg!(windows) || {
                #[cfg(target_os = "linux")]
                {
                    self.connection.is_some()
                }
                #[cfg(not(target_os = "linux"))]
                {
                    false
                }
            },
            display: cfg!(windows),
            backend: if cfg!(windows) {
                "Windows · Execution State"
            } else if cfg!(target_os = "linux") {
                "Linux · logind (solo sistema)"
            } else {
                "Sistema non supportato"
            }
            .into(),
        }
    }

    pub fn set_mode(&mut self, mode: AwakeMode) -> Result<(), String> {
        #[cfg(windows)]
        {
            use windows_sys::Win32::System::Power::{
                ES_CONTINUOUS, ES_DISPLAY_REQUIRED, ES_SYSTEM_REQUIRED, SetThreadExecutionState,
            };
            let flags = match mode {
                AwakeMode::Off => ES_CONTINUOUS,
                AwakeMode::System => ES_CONTINUOUS | ES_SYSTEM_REQUIRED,
                AwakeMode::Display => ES_CONTINUOUS | ES_SYSTEM_REQUIRED | ES_DISPLAY_REQUIRED,
            };
            // SAFETY: no pointers; called on the owning worker thread for its whole lifetime.
            if unsafe { SetThreadExecutionState(flags) } == 0 {
                return Err(
                    "Windows non ha accettato la richiesta di mantenimento in attività.".into(),
                );
            }
            Ok(())
        }
        #[cfg(target_os = "linux")]
        {
            if mode == AwakeMode::Off {
                self.inhibitor = None;
                return Ok(());
            }
            if mode == AwakeMode::Display {
                return Err("Il controllo del display Linux non è ancora disponibile.".into());
            }
            let connection = self
                .connection
                .as_ref()
                .ok_or("Bus di sistema non disponibile")?;
            let proxy = zbus::blocking::Proxy::new(
                connection,
                "org.freedesktop.login1",
                "/org/freedesktop/login1",
                "org.freedesktop.login1.Manager",
            )
            .map_err(|e| format!("logind non disponibile: {e}"))?;
            let handle: zbus::zvariant::OwnedFd = proxy
                .call(
                    "Inhibit",
                    &(
                        "sleep",
                        "MonitorAwake",
                        "Timer o richiesta manuale",
                        "block",
                    ),
                )
                .map_err(|e| format!("Impossibile inibire la sospensione: {e}"))?;
            self.inhibitor = Some(handle);
            Ok(())
        }
        #[cfg(not(any(windows, target_os = "linux")))]
        {
            if mode == AwakeMode::Off {
                Ok(())
            } else {
                Err("Piattaforma non supportata.".into())
            }
        }
    }
}

impl Drop for NativeAwake {
    fn drop(&mut self) {
        let _ = self.set_mode(AwakeMode::Off);
    }
}
