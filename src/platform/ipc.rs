use std::sync::mpsc;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IconEventData {
    pub uid: Option<u32>,
    pub window_handle: Option<isize>,
    pub guid: Option<uuid::Uuid>,
    pub tooltip: Option<String>,
    pub icon_handle: Option<isize>,
    pub callback_message: Option<u32>,
    pub version: Option<u32>,
    pub is_visible: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[allow(clippy::enum_variant_names)]
pub enum Win32TrayEvent {
    IconAdd { data: IconEventData },
    IconUpdate { data: IconEventData },
    IconRemove { data: IconEventData },
}

#[derive(Debug)]
pub struct TrayIpcServer {
    _handle: Option<std::thread::JoinHandle<()>>,
}

impl TrayIpcServer {
    pub fn start() -> (Self, mpsc::Receiver<Win32TrayEvent>) {
        let (tx, rx) = mpsc::channel();
        let handle = std::thread::Builder::new()
            .name("nl-tray-ipc".to_string())
            .spawn(move || {
                Self::server_loop(tx);
            })
            .ok();

        (Self { _handle: handle }, rx)
    }

    fn pipe_name() -> String {
        let session_id = current_session_id();
        format!(r"\\.\pipe\nl-tray-{}", session_id)
    }

    fn server_loop(tx: mpsc::Sender<Win32TrayEvent>) {
        use interprocess::os::windows::named_pipe::{pipe_mode, PipeListenerOptions};
        use std::io::{Read, Write};
        use std::path::Path;

        let pipe_name = Self::pipe_name();

        loop {
            let listener = match PipeListenerOptions::new()
                .path(Path::new(&pipe_name))
                .create_duplex::<pipe_mode::Bytes>()
            {
                Ok(l) => l,
                Err(e) => {
                    eprintln!("[nl-tray] Failed to create pipe listener: {}", e);
                    std::thread::sleep(std::time::Duration::from_secs(1));
                    continue;
                }
            };

            for stream_result in listener.incoming() {
                let mut stream = match stream_result {
                    Ok(s) => s,
                    Err(e) => {
                        eprintln!("[nl-tray] Pipe accept error: {}", e);
                        continue;
                    }
                };
                let tx = tx.clone();
                std::thread::spawn(move || {
                    let mut buf = vec![0u8; 65536];
                    loop {
                        match stream.read(&mut buf) {
                            Ok(0) => break,
                            Ok(n) => {
                                if let Ok(event) = serde_json::from_slice::<Win32TrayEvent>(&buf[..n]) {
                                    let _ = tx.send(event);
                                }
                                let _ = stream.write_all(b"ok");
                            }
                            Err(_) => break,
                        }
                    }
                });
            }
        }
    }
}

fn current_session_id() -> u32 {
    unsafe {
        let mut sid = 0u32;
        let _ = windows::Win32::System::RemoteDesktop::ProcessIdToSessionId(
            std::process::id(),
            &mut sid,
        );
        sid
    }
}
