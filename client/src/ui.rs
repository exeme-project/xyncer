use eframe::egui;
use std::sync::{Arc, Mutex};

use crate::client;
use crate::session;

pub struct Xyncer {
    pub session: Arc<std::sync::Mutex<session::Session>>,
}

impl Default for Xyncer {
    fn default() -> Self {
        Self {
            session: Arc::new(Mutex::new(session::Session {
                authenticated: false,
                connected: false,
                error: anyhow::Error::msg(""),
                password: String::new(),
                server_address: String::new(),
            })),
        }
    }
}

impl eframe::App for Xyncer {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let session_clone_for_client = self.session.clone();
        let session_clone_for_error = self.session.clone();

        let mut session = self.session.lock().unwrap(); // Lock the mutex

        egui::CentralPanel::default().show(ctx, |ui| {
            if session.authenticated {
            } else if session.connected {
                ui.heading(egui::RichText::new("Xyncer").size(30.0));

                ui.horizontal(|ui| {
                    ui.label("Password:");
                    ui.text_edit_singleline(&mut session.password);
                });

                let error = session.error.to_string();

                if !error.is_empty() {
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("Error: ")
                                .color(egui::Color32::from_rgb(255, 105, 97)),
                        );
                        ui.label(error);
                    });
                }

                if ui.button("Authenticate").clicked() {}
            } else {
                ui.heading(egui::RichText::new("Xyncer").size(30.0));

                ui.horizontal(|ui| {
                    ui.label("Server Address:");
                    ui.text_edit_singleline(&mut session.server_address);
                });

                let error = session.error.to_string();

                if !error.is_empty() {
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("Error: ")
                                .color(egui::Color32::from_rgb(255, 105, 97)),
                        );
                        ui.label(error);
                    });
                }

                if ui.button("Connect").clicked() {
                    let server_address = session.server_address.clone();

                    std::mem::drop(session); // Unlock the mutex

                    tokio::spawn(async move {
                        match client::start_client(&server_address, session_clone_for_client).await
                        {
                            Ok(_) => (),
                            Err(e) => {
                                let mut session = session_clone_for_error.lock().unwrap();

                                session.error = e;
                            }
                        }
                    });
                }
            }
        });
    }
}
