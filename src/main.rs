#![windows_subsystem = "windows"] //hide console
mod config;
mod osc_thread;

use eframe::egui;

use std::fs;
use std::sync::mpsc;
use std::thread::{self};

//define data structure
#[derive(Clone)] //TODO: awkward name
struct Configs {
    enabled: bool,
    hints_enabled: bool,
    ip_address: String,
    height: f32,
    height_offset: f32,
    hip_enabled: bool,
    left_foot_enabled: bool,
    right_foot_enabled: bool,
    locked_to_headset: bool,
}

impl Configs {
    fn default() -> Self {
        Configs {
            enabled: false,
            hints_enabled: true,
            ip_address: "127.0.0.1:9000".to_string(),
            height: 0.0,
            height_offset: 0.0,
            hip_enabled: true,
            locked_to_headset: false,
            left_foot_enabled: true,
            right_foot_enabled: true,
        }
    }
}

fn main() {
    //handle app data
    let configs;
    //if RUSTVAIL-autoload.cfg exists, load it
    if fs::metadata("RUSTVAIL-autoload.cfg").is_ok() {
        configs = config::parse("RUSTVAIL-autoload.cfg");
    } else {
        configs = Configs::default();
    }

    //create a channel so the threads can communicate
    let (tx, rx) = mpsc::channel::<Configs>();

    //spawn second thread to handle OSC messages
    let osc_thread_info = thread::spawn(move || {
        osc_thread::thread(rx);
    });

    // Initialize egui window
    let icon =
        eframe::icon_data::from_png_bytes(include_bytes!("../assets/RVO.png")).unwrap_or_default();
    let window_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([490.0, 440.0])
            .with_icon(icon),
        ..Default::default()
    };
    let _ = eframe::run_native(
        format!("RustVail OSC Beta v{}", env!("CARGO_PKG_VERSION")).as_str(),
        window_options,
        Box::new(|cc| {
            Ok(Box::new(GuiApp::new(
                cc,
                "".to_string(),
                configs,
                osc_thread_info,
                tx,
            )))
        }),
    );

    //TODO: awkward name
    struct GuiApp {
        configs: Configs,
        osc_thread_info: thread::JoinHandle<()>,
        thread_sender: mpsc::Sender<Configs>,
        ui_ip_address: String,
    }
    impl GuiApp {
        fn new(
            cc: &eframe::CreationContext<'_>,
            ui_ip_address: String,
            thread_data: Configs,
            thread_info: thread::JoinHandle<()>,
            thread_sender: mpsc::Sender<Configs>,
        ) -> Self {
            cc.egui_ctx.set_zoom_factor(2.0);
            Self {
                configs: thread_data,
                osc_thread_info: thread_info,
                thread_sender,
                ui_ip_address,
            }
        }
    }
    impl eframe::App for GuiApp {
        fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
            egui::CentralPanel::default().show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                        //Heading
                        ui.heading("RustVail OSC");
                        if self.osc_thread_info.is_finished() {
                            ui.label("Worker thread died, probably something bad happened, please try restarting the app.\n\nUsually this is caused by a bad config file or a bad ip address");
                        }
                        ui.checkbox(&mut self.configs.enabled, "Enabled");
                        //Settings
                        ui.collapsing("Settings", |ui| {
                            ui.checkbox(&mut self.configs.hints_enabled, "Hints");
                            hint(ui, self.configs.hints_enabled, "Don't forget to save every time you make a change you want to keep!");
                            if ui.button("Save current config as default").clicked() { //TODO: for the love of god, make this a checkbox or a switch
                                config::save("RUSTVAIL-autoload.cfg", &self.configs);
                            }
                            ui.horizontal(|ui| {
                            if ui.button("Remove default config").clicked() {
                                config::delete_config();
                            }
                            if ui.button("Reset app").clicked() {
                                self.configs = Configs::default();
                            }
                            });
                            ui.collapsing("Connection", |ui| {
                                hint(ui, self.configs.hints_enabled, "Set the IP address of the computer/quest running VRChat, change the :9000 at the end only if you know what you're doing!");
                                ui.horizontal(|ui| {
                                ui.label("IP address:");
                                ui.add(egui::TextEdit::singleline(&mut self.ui_ip_address).hint_text(&self.configs.ip_address));
                            });
                            if ui.button("Set new IP").clicked() {
                                if !self.ui_ip_address.is_empty() {
                                    self.configs.ip_address = self.ui_ip_address.clone();
                                    self.ui_ip_address = "".to_string();
                                }
                            }
                            });
                            ui.collapsing("About", |ui| {
                                hint(ui, self.configs.hints_enabled, "Report any bugs or issues to the github page!");
                                ui.label(format!("RVOSC Beta v{}", env!("CARGO_PKG_VERSION")));
                                ui.label("Made by Aelx with <3");
                                ui.label("Licensed under MIT License Copyright (c) 2025 Aeelx");
                                ui.hyperlink("https://github.com/Aeelx/RustVail-OSC");
                            });
                        });
                    });
                    ui.separator();
                    //Body
                    ui.collapsing("Offsets", |ui| {
                        hint(ui, self.configs.hints_enabled, "You can drag the number to quickly change the height of the trackers");
                        ui.horizontal(|ui| {
                            ui.label("Current height offset:");
                            ui.add(egui::DragValue::new(&mut self.configs.height).speed(0.015));
                        });
                        if ui.button("Reset height to default").clicked() {
                            self.configs.height = 0.0;
                        }
                    });
                    ui.collapsing("Enabled trackers", |ui| {
                        hint(ui, self.configs.hints_enabled, "Enable the head tracker to lock the OSC trackers to your headset");
                        ui.checkbox(&mut self.configs.locked_to_headset, "Head");
                        ui.checkbox(&mut self.configs.hip_enabled, "Hip");
                        ui.checkbox(&mut self.configs.left_foot_enabled, "Left foot");
                        ui.checkbox(&mut self.configs.right_foot_enabled, "Right foot");
                    });
                    ui.separator();
                });
                //send thread data to thread and ignore errors because sometimes the thread has an heart attack and that's okay
                let _ = self.thread_sender.send(self.configs.clone());
            });
        }
    }
}

fn hint(ui: &mut egui::Ui, hints_enabled: bool, hint_text: &str) {
    if hints_enabled {
        ui.add(
            egui::Label::new(
                egui::RichText::new(format!("> {}", hint_text))
                    .small()
                    .weak(),
            )
            .selectable(false),
        );
    }
}
