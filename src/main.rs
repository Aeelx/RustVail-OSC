#![windows_subsystem = "windows"] //hide console

mod osc_thread;

use eframe::egui;

use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::sync::mpsc;
use std::thread::{self};

//define data structure
#[derive(Clone)] //TODO: awkward name
struct ThreadData {
    enabled: bool,
    ip_address: String,
    height: f32,
    height_offset: f32,
    hip_enabled: bool,
    left_foot_enabled: bool,
    right_foot_enabled: bool,
    locked_to_headset: bool,
}

impl ThreadData {
    fn default() -> Self {
        ThreadData {
            enabled: false,
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
    let thread_data;
    //if RUSTVAIL-autoload.cfg exists, load it
    if fs::metadata("RUSTVAIL-autoload.cfg").is_ok() {
        thread_data = parse_config("RUSTVAIL-autoload.cfg");
    } else {
        thread_data = ThreadData::default();
    }

    //create a channel so the threads can communicate
    let (tx, rx) = mpsc::channel::<ThreadData>();

    //spawn second thread to handle OSC messages
    let thread_info = thread::spawn(move || {
        osc_thread::thread(rx);
    });

    // Initialize egui window
    let icon = eframe::icon_data::from_png_bytes(include_bytes!("../assets/RVO.png")).unwrap();
    let window_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([490.0, 540.0])
            .with_icon(icon),
        ..Default::default()
    };
    let _ = eframe::run_native(
        format!("RustVail OSC Beta v{}", env!("CARGO_PKG_VERSION")).as_str(),
        window_options,
        Box::new(|cc| {
            Ok(Box::new(GuiApp::new(
                cc,
                thread_data.ip_address.clone(),
                thread_data,
                thread_info,
                tx,
            )))
        }),
    );

    //TODO: awkward name
    struct GuiApp {
        thread_data: ThreadData,
        thread_info: thread::JoinHandle<()>,
        thread_sender: mpsc::Sender<ThreadData>,
        ui_ip_address: String,
    }

    impl GuiApp {
        fn new(
            cc: &eframe::CreationContext<'_>,
            ui_ip_address: String,
            thread_data: ThreadData,
            thread_info: thread::JoinHandle<()>,
            thread_sender: mpsc::Sender<ThreadData>,
        ) -> Self {
            cc.egui_ctx.set_zoom_factor(2.0);
            Self {
                thread_data,
                thread_info,
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
                        ui.heading("RustVail OSC");
                        if self.thread_info.is_finished() {
                            ui.label("Worker thread died, probably something bad happened, please try restarting the app.\n\nUsually this is caused by a bad config file or a bad ip address");
                        }
                        ui.checkbox(&mut self.thread_data.enabled, "Enabled");
                        ui.collapsing("Config", |ui| {
                            if ui.button("Save config").clicked() { //TODO: for the love of god, make this a checkbox or a switch
                                save_config("RUSTVAIL-autoload.cfg", &self.thread_data);
                            }
                            ui.horizontal(|ui| {
                            if ui.button("Remove saved configs").clicked() {
                                //delete autoload config
                                let result = std::fs::remove_file("RUSTVAIL-autoload.cfg");
                                match result {
                                    Ok(_) => {
                                        // file removed
                                    }
                                    Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                                        // ignore if file not found
                                    }
                                    Err(e) => {
                                        panic!("Failed to remove file: {:?}", e);
                                    }
                                }
                            }
                            if ui.button("Reset app").clicked() {
                                self.thread_data = ThreadData::default();
                            }
                            });
                            //ui.horizontal(|ui| {
                            //    if ui.button("Manual save").clicked() {
                            //        save_config("RUSTVAIL-config.cfg", &self.thread_data);
                            //    }
                            //    if ui.button("Manual load").clicked() {
                            //        self.thread_data = parse_config("RUSTVAIL-config.cfg");
                            //    }
                            //});
                        });
                        ui.collapsing("About", |ui| {
                            ui.label(format!("RVOSC Beta v{}", env!("CARGO_PKG_VERSION")));
                            ui.label("Made by Aelx with <3");
                            ui.label("Licensed under MIT License Copyright (c) 2025 Aeelx");
                            ui.label("https://github.com/Aeelx/RustVail-OSC");
                        });
                    });
                    ui.separator();
                    ui.collapsing("Connection", |ui| {
                        ui.label("IP address:");
                        ui.text_edit_singleline(&mut self.ui_ip_address);
                        if ui.button("Set new IP").clicked() {
                            self.thread_data.ip_address = self.ui_ip_address.clone();
                        }
                    });
                    ui.collapsing("Offsets", |ui| {
                        ui.label(format!(
                            "Current height:'{}', offset:{}",
                            self.thread_data.height, self.thread_data.height_offset
                        ));
                        ui.add(
                            egui::Slider::new(&mut self.thread_data.height, -3.0..=3.0)
                                .text("Height"),
                        );
                        if ui.button("Apply height to height offset").clicked() {
                            self.thread_data.height_offset += self.thread_data.height;
                            self.thread_data.height = 0.0;
                        }
                    });
                    ui.collapsing("Enabled trackers", |ui| {
                        ui.checkbox(&mut self.thread_data.locked_to_headset, "Head (Lock OSC trackers to headset)");
                        ui.checkbox(&mut self.thread_data.hip_enabled, "Hip");
                        ui.checkbox(&mut self.thread_data.left_foot_enabled, "Left foot");
                        ui.checkbox(&mut self.thread_data.right_foot_enabled, "Right foot");
                    });
                    ui.separator();
                });
                //send thread data to thread and ignore errors because sometimes the thread has an heart attack
                let _ = self.thread_sender.send(self.thread_data.clone());
            });
        }
    }
}

//TODO: better error handling
//parse extremely simple config file
fn parse_config(filename: &str) -> ThreadData {
    //read file
    let contents = match fs::read_to_string(filename) {
        Ok(contents) => contents,
        Err(err) => {
            println!("Failed to read config file: {:?}", err);
            return ThreadData::default();
        }
    };
    let mut config = HashMap::new();

    //make a hashmap from key=value pairs
    for line in contents.lines() {
        if let Some((key, value)) = line.split_once('=') {
            config.insert(key.trim().to_string(), value.trim().to_string());
        }
    }

    let thread_data = ThreadData {
        enabled: config
            .get("enabled")
            .unwrap()
            .parse::<bool>()
            .unwrap_or_default(),
        ip_address: config
            .get("ip_address")
            .unwrap()
            .parse::<String>()
            .unwrap_or_default(),
        height: config
            .get("height")
            .unwrap()
            .parse::<f32>()
            .unwrap_or_default(),
        height_offset: config
            .get("height_offset")
            .unwrap()
            .parse::<f32>()
            .unwrap_or_default(),
        locked_to_headset: config
            .get("locked_to_headset")
            .unwrap()
            .parse::<bool>()
            .unwrap_or_default(),
        hip_enabled: config
            .get("hip_enabled")
            .unwrap()
            .parse::<bool>()
            .unwrap_or_default(),
        left_foot_enabled: config
            .get("left_foot_enabled")
            .unwrap()
            .parse::<bool>()
            .unwrap_or_default(),
        right_foot_enabled: config
            .get("right_foot_enabled")
            .unwrap()
            .parse::<bool>()
            .unwrap_or_default(),
    };

    return thread_data;
}

//TODO: error handling
fn save_config(filename: &str, thread_data: &ThreadData) {
    let mut file = fs::File::create(filename).expect("Failed to create config file");

    //make a hashmap of key=value pairs TODO: use thread_data struct to define keys
    let mut config = HashMap::new();
    config.insert("enabled".to_string(), thread_data.enabled.to_string());
    config.insert("height".to_string(), thread_data.height.to_string());
    config.insert(
        "height_offset".to_string(),
        thread_data.height_offset.to_string(),
    );
    config.insert(
        "hip_enabled".to_string(),
        thread_data.hip_enabled.to_string(),
    );
    config.insert(
        "left_foot_enabled".to_string(),
        thread_data.left_foot_enabled.to_string(),
    );
    config.insert(
        "right_foot_enabled".to_string(),
        thread_data.right_foot_enabled.to_string(),
    );
    config.insert(
        "locked_to_headset".to_string(),
        thread_data.locked_to_headset.to_string(),
    );

    //write the hashmap to the file
    writeln!(
        file,
        "#RustVail server properties ^^\n#Delete this if you are having issues"
    )
    .expect("Failed to write to config file");
    for (key, value) in config {
        writeln!(file, "{}={}", key, value).expect("Failed to write to config file");
    }
}
