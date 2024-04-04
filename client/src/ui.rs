use eframe::egui;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::{client::start_client, session};

pub struct Xyncer {
    pub payload_sender: flume::Sender<xyncer_share::Payload>,
    pub payload_receiver: flume::Receiver<xyncer_share::Payload>,

    pub session_data_guard: Arc<RwLock<session::Session>>,
}

impl Default for Xyncer {
    fn default() -> Self {
        let (payload_sender, payload_receiver) = flume::unbounded();

        let session_data = session::Session {
            authenticated: false,
            connected: false,
            error: None,
            password: String::new(),
            server_address: String::new(),
        };

        Xyncer {
            payload_sender: payload_sender,
            payload_receiver: payload_receiver,
            session_data_guard: Arc::new(RwLock::new(session_data)),
        }
    }
}

impl eframe::App for Xyncer {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Obtain a read lock on the session data
        let session_data = self.session_data_guard.try_read().unwrap();

        egui::CentralPanel::default().show(ctx, |ui| {
            if session_data.authenticated {
            } else if session_data.connected {
            } else {
                // Drop the read lock on the session data
                drop(session_data);

                let mut session_data: tokio::sync::RwLockWriteGuard<session::Session>;

                // Loop until we obtain a write lock on the session data
                loop {
                    // Try to obtain a write lock on the session data
                    let try_session_data = self.session_data_guard.try_write();

                    if try_session_data.is_ok() {
                        // Set the session data
                        session_data = try_session_data.unwrap();

                        break;
                    } else {
                        std::thread::sleep(std::time::Duration::from_millis(100));
                    }
                }

                ui.heading("Xyncer");

                ui.horizontal(|ui| {
                    ui.label("Server Address:");
                    ui.text_edit_singleline(&mut session_data.server_address);
                });

                if ui.button("Connect").clicked() {
                    // Remove write lock on session data
                    drop(session_data);

                    // Clone the session data guard and payload sender/receiver
                    let session_data_clone = self.session_data_guard.clone();
                    let payload_sender_clone = self.payload_sender.clone();
                    let payload_receiver_clone = self.payload_receiver.clone();

                    // Start the client
                    tokio::spawn(async move {
                        if let Err(e) = start_client(
                            session_data_clone,
                            payload_receiver_clone,
                            payload_sender_clone,
                        )
                        .await
                        {
                            log::error!("Error running client: {}", e);
                        }
                    });
                }
            }
        });
    }
}
