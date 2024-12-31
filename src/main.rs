
#![windows_subsystem = "windows"] //hide console

use eframe::egui;

extern crate nannou_osc;
use nannou_osc::Type::Float;
use nannou_osc::{Message, Sender};

use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::sync::mpsc;
use std::thread::{self};

//define data structure
#[derive(Clone)] //TODO: awkward name
struct ThreadData {
    enabled: bool,
    height: f32,
    height_offset: f32,
    hip_enabled: bool,
    locked_to_headset: bool,
}

impl ThreadData {
    fn default() -> Self {
        ThreadData {
            enabled: false,
            height: 0.0,
            height_offset: 0.0,
            hip_enabled: true,
            locked_to_headset: false,
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
    thread::spawn(move || {
        let osc_tx = Sender::bind()
            .expect("Couldn't bind to default socket")
            .connect("127.0.0.1:9000") //TODO: add way to set ip so you can run it on a computer if you're on quest
            .expect("Couldn't connect to socket at address");

        //hang thread until we get the first message from the first render
        let mut data = rx.recv().unwrap();

        //loop with timeout so we can send a value at least once a second even if the other thread stops/slows down rendering
        loop {
            match rx.recv_timeout(std::time::Duration::from_millis(300)) {
                Ok(value) => data = value,
                Err(mpsc::RecvTimeoutError::Timeout) => (),
                Err(err) => panic!("Unexpected error: {:?}", err),
            };

            osc_loop(&osc_tx, &data)
        }
    });

    // Initialize egui window
    let icon = eframe::icon_data::from_png_bytes(include_bytes!("../assets/RVO.png")).unwrap();
    let window_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([490.0, 440.0])
            .with_icon(icon),
        ..Default::default()
    };
    let _ = eframe::run_native(
        "RustVail OSC Beta v0.3",
        window_options,
        Box::new(|cc| Ok(Box::new(GuiApp::new(cc, thread_data, tx)))),
    );

    //TODO: awkward name
    struct GuiApp {
        thread_data: ThreadData,
        thread_sender: mpsc::Sender<ThreadData>,
    }

    impl GuiApp {
        fn new(
            cc: &eframe::CreationContext<'_>,
            thread_data: ThreadData,
            thread_sender: mpsc::Sender<ThreadData>,
        ) -> Self {
            cc.egui_ctx.set_zoom_factor(2.0);
            Self {
                thread_data,
                thread_sender,
            }
        }
    }


    //todo: too much logic in one place
    impl eframe::App for GuiApp {
        fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
            egui::CentralPanel::default().show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                        ui.heading("RustVail OSC");
                        ui.checkbox(&mut self.thread_data.enabled, "Enabled");
                        ui.collapsing("Save/load config", |ui| {
                            ui.label("Delete the config files if you're having issues");
                            if ui.button("Save and auto load on next boot").clicked() {
                                save_config("RUSTVAIL-autoload.cfg", &self.thread_data);
                            }
                            if ui.button("Reset app and disable auto load").clicked() {
                                self.thread_data = ThreadData::default();
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
                            ui.separator();
                            ui.horizontal(|ui| {
                                if ui.button("Manual save").clicked() {
                                    save_config("RUSTVAIL-config.cfg", &self.thread_data);
                                }
                                if ui.button("Manual load").clicked() {
                                    self.thread_data = parse_config("RUSTVAIL-config.cfg");
                                }
                            });
                        });
                        ui.collapsing("About", |ui| {
                            ui.label("RVOSC Beta v0.3"); 
                            ui.label("Made by Aelx with <3");
                            ui.label("Licensed under MIT License Copyright (c) 2024 Aeelx");
                            ui.label("https://github.com/Aeelx/RustVail-OSC");
                        });
                    });
                    ui.separator();
                    ui.label(format!(
                        "Current height:'{}', offset:{}",
                        self.thread_data.height, self.thread_data.height_offset
                    ));
                    ui.add(
                        egui::Slider::new(&mut self.thread_data.height, -3.0..=3.0).text("Height"),
                    );
                    if ui.button("Apply height to height offset").clicked() {
                        self.thread_data.height_offset += self.thread_data.height;
                        self.thread_data.height = 0.0;
                    }
                    ui.checkbox(
                        &mut self.thread_data.locked_to_headset,
                        "Lock trackers to headset",
                    );
                    ui.separator();
                    ui.checkbox(&mut self.thread_data.hip_enabled, "Hip tracker");
                });
                self.thread_sender.send(self.thread_data.clone()).unwrap();
            });
        }
    }
}

fn osc_loop(osc_tx: &Sender<nannou_osc::Connected>, thread_data: &ThreadData) {
    //check if we should send anything
    if !thread_data.enabled {
        return;
    }

    //create messages to send
    let left_foot = Message {
        addr: "/tracking/trackers/1/position".to_string(),
        args: [Float(-0.1), Float(0.0 + thread_data.height), Float(0.0)].to_vec(),
    };
    let right_foot = Message {
        addr: "/tracking/trackers/2/position".to_string(),
        args: [Float(0.1), Float(0.0 + thread_data.height), Float(0.0)].to_vec(),
    };
    let hip = Message {
        addr: "/tracking/trackers/3/position".to_string(),
        args: [Float(0.0), Float(0.9 + thread_data.height), Float(0.0)].to_vec(),
    };
    let head_position = Message {
        addr: "/tracking/trackers/head/position".to_string(),
        args: [Float(0.0), Float(1.75 + thread_data.height), Float(0.0)].to_vec(),
    };
    let head_rotation = Message {
        addr: "/tracking/trackers/head/rotation".to_string(),
        args: [Float(0.0), Float(0.0), Float(0.0)].to_vec(),
    };

    //send only the messages that we should
    osc_tx.send(left_foot).unwrap();
    osc_tx.send(right_foot).unwrap();
    if thread_data.hip_enabled {
        osc_tx.send(hip).unwrap();
    }
    if thread_data.locked_to_headset {
        osc_tx.send(head_position).unwrap();
        osc_tx.send(head_rotation).unwrap();
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
        hip_enabled: config
            .get("hip_enabled")
            .unwrap()
            .parse::<bool>()
            .unwrap_or_default(),
        locked_to_headset: config
            .get("locked_to_headset")
            .unwrap()
            .parse::<bool>()
            .unwrap_or_default(),
    };

    return thread_data;
}

//TODO: error handling
fn save_config(filename: &str, thread_data: &ThreadData) {
    let mut file = fs::File::create(filename).expect("Failed to create config file");

    //make a hashmap of key=value pairs
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
