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
        use egui::special_emojis::{GITHUB, OS_APPLE, OS_LINUX, OS_WINDOWS};

        // Obtain a read lock on the session data
        let session_data = self.session_data_guard.try_read().unwrap();

        egui::CentralPanel::default().show(ctx, |ui| {
            if session_data.authenticated {
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
                    }
                }

                ui.heading("xyncer");

                ui.label(format!(
                    "Xyncer is a tool that allows you to seamlessly share windows between devices. It runs natively on {}{}{}. To get started, enter the server below, and if connection is successful, you will be prompted to authenticate.",
                    OS_WINDOWS, OS_LINUX, OS_APPLE
                ));

                ui.add_space(12.0);

                ui.heading("Authentication");

                ui.horizontal(|ui| {
                    ui.label("Server Address:");
                    ui.text_edit_singleline(&mut session_data.server_address);
                });

                if session_data.connected {
                    ui.horizontal(|ui| {
                        ui.label("Password:");
                        ui.text_edit_singleline(&mut session_data.password);
                    });
                }

                if session_data.error.is_some() {
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("Error running client:")
                                .color(egui::Color32::from_rgb(255, 105, 97)),
                        );
                        ui.label(session_data.error.as_ref().unwrap());
                    });
                }

                // Drop the write lock on the session data
                drop(session_data);

                if ui.button("Connect").clicked() {
                    // Clone the session data guard and payload sender/receiver
                    let session_data_guard_clone = self.session_data_guard.clone();
                    let payload_sender_clone = self.payload_sender.clone();
                    let payload_receiver_clone = self.payload_receiver.clone();

                    // Start the client
                    tokio::spawn(async move {
                        let session_data_guard_clone_clone = session_data_guard_clone.clone();

                        if let Err(e) = start_client(
                            session_data_guard_clone,
                            payload_receiver_clone,
                            payload_sender_clone,
                        )
                        .await
                        {
                            log::error!("Error running client: {}", e);

                            // Obtain a write lock on the session data
                            let mut session_data =
                                session_data_guard_clone_clone.try_write().unwrap();

                            // Set the error message
                            session_data.error = e.to_string().into();

                            // Drop the write lock on the session data
                            drop(session_data);
                        }
                    });
                }

                ui.add_space(12.0);

                ui.heading("Links");

                ui.hyperlink_to(format!("{} xyncer on GitHub", GITHUB), "https://github.com/exeme-project/xyncer");
                ui.hyperlink_to(format!("{} exeme-project on GitHub", GITHUB), "https://github.com/exeme-project");
                ui.hyperlink_to("exeme-project documentation", "https://exeme-project.github.io/");

                ui.add_space(12.0);

                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    ui.label("xyncer is primarily developed by ");
                    ui.hyperlink_to("skifli", "https://github.com/skifli");
                    ui.label(" through the ");
                    ui.hyperlink_to("exeme-project", "https://github.com/exeme-project");
                    ui.label(" with help through other ");
                    ui.hyperlink_to("contributors", "https://github.com/exeme-project/.github/blob/main/CONTRIBUTORS.md");
                    ui.label(".");
                });
            }
        });
    }
}
