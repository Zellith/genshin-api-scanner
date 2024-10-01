#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

use eframe::egui;
use eframe::egui::{Align, Layout};
use reqwest;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Deserialize)]
struct ApiResponse {
    retcode: i32,
    message: String,
    data: Data,
}

#[derive(Deserialize)]
struct Data {
    game_packages: Vec<GamePackage>,
}

#[derive(Deserialize)]
struct GamePackage {
    #[allow(dead_code)]
    game: Game,
    main: Main,
}

#[derive(Deserialize)]
struct Game {
    #[allow(dead_code)]
    id: String,
    #[allow(dead_code)]
    biz: String,
}

#[derive(Deserialize)]
struct Main {
    major: Option<Major>,
}

#[derive(Deserialize)]
struct Major {
    #[allow(dead_code)]
    version: String,
    game_pkgs: Vec<Package>,
    audio_pkgs: Vec<AudioPackage>,
}

#[derive(Deserialize)]
struct Package {
    url: String,
    #[allow(dead_code)]
    md5: String,
    size: String,
    decompressed_size: String,
}

#[derive(Deserialize)]
struct AudioPackage {
    language: String,
    url: String,
    #[allow(dead_code)]
    md5: String,
    size: String,
    decompressed_size: String,
}

struct GenshinApp {
    data: Arc<Mutex<String>>,
    formatted_message: Arc<Mutex<String>>,
}

impl Default for GenshinApp {
    fn default() -> Self {
        Self {
            data: Arc::new(Mutex::new(
                "Press 'Fetch Data' to get the latest data.".to_string(),
            )),
            formatted_message: Arc::new(Mutex::new(String::new())),
        }
    }
}

impl eframe::App for GenshinApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let data_clone = Arc::clone(&self.data);
        let formatted_message_clone = Arc::clone(&self.formatted_message);

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
                // Fetch Data Button
                if ui.button("Fetch Data").clicked() {
                    // Clear the formatted message
                    {
                        let mut message_lock = formatted_message_clone.lock().unwrap();
                        *message_lock = String::new();
                    }

                    let data_clone = Arc::clone(&data_clone);
                    std::thread::spawn(move || {
                        let result = fetch_and_process_data();
                        let mut data_lock = data_clone.lock().unwrap();
                        match result {
                            Ok(data) => {
                                *data_lock = data;
                            }
                            Err(err) => {
                                *data_lock = format!("Error fetching data: {}", err);
                            }
                        }
                    });
                }

                // Convert to Message Button
                if ui.button("Convert to Message").clicked() {
                    let data_clone = Arc::clone(&data_clone);
                    let formatted_message_clone = Arc::clone(&formatted_message_clone);
                    let data = data_clone.lock().unwrap().clone();
                    let formatted_message = convert_to_message(&data);
                    let mut message_lock = formatted_message_clone.lock().unwrap();
                    *message_lock = formatted_message;
                }

                // Copy to Clipboard Button
                if ui.button("Copy to Clipboard").clicked() {
                    let message_lock = formatted_message_clone.lock().unwrap();
                    let data_lock = data_clone.lock().unwrap();
                    let text_to_copy = if !message_lock.is_empty() {
                        message_lock.clone()
                    } else {
                        data_lock.clone()
                    };
                    ctx.output_mut(|o| o.copied_text = text_to_copy);
                }
            });

            ui.separator();

            // Display the data or the formatted message
            let message = formatted_message_clone.lock().unwrap().clone();
            if !message.is_empty() {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.label(&message);
                });
            } else {
                let data = data_clone.lock().unwrap().clone();
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.label(&data);
                });
            }
        });
    }
}

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Genshin Package Viewer",
        native_options,
        Box::new(|_cc| Box::new(GenshinApp::default())),
    )
}

// Helper function to fetch and process data
fn fetch_and_process_data() -> Result<String, String> {
    let url = "https://sg-hyp-api.hoyoverse.com/hyp/hyp-connect/api/getGamePackages?game_ids[]=gopR6Cufr3&launcher_id=VYTpXlbWo8";

    let response = reqwest::blocking::get(url)
        .map_err(|e| format!("Request error: {}", e))?
        .text()
        .map_err(|e| format!("Response text error: {}", e))?;

    let api_response: ApiResponse = serde_json::from_str(&response)
        .map_err(|e| format!("JSON parse error: {}", e))?;

    if api_response.retcode != 0 {
        return Err(format!("API returned an error: {}", api_response.message));
    }

    let mut output = String::new();

    for game_package in api_response.data.game_packages {
        if let Some(major) = game_package.main.major {
            // Game Packages
            output.push_str("Game Packages:\n");
            for (index, pkg) in major.game_pkgs.iter().enumerate() {
                let part_number = index + 1;
                let size_gb = bytes_to_gb(&pkg.size);
                let decompressed_size_gb = bytes_to_gb(&pkg.decompressed_size);
                output.push_str(&format!("[Part {}]\n", part_number));
                output.push_str(&format!("[URL] {}\n", pkg.url));
                output.push_str(&format!("[Size] {:.2}GB\n", size_gb));
                output.push_str(&format!(
                    "[Decompressed Size] {:.2}GB\n\n",
                    decompressed_size_gb
                ));
            }

            // Audio Packages
            output.push_str("Audio Packages:\n");
            for audio_pkg in major.audio_pkgs {
                let size_gb = bytes_to_gb(&audio_pkg.size);
                let decompressed_size_gb = bytes_to_gb(&audio_pkg.decompressed_size);
                output.push_str(&format!("[Language] {}\n", audio_pkg.language));
                output.push_str(&format!("[URL] {}\n", audio_pkg.url));
                output.push_str(&format!("[Size] {:.2}GB\n", size_gb));
                output.push_str(&format!(
                    "[Decompressed Size] {:.2}GB\n\n",
                    decompressed_size_gb
                ));
            }
        } else {
            output.push_str("No major version data available.\n");
        }
    }

    Ok(output)
}

// Helper function to convert size in bytes (as string) to gigabytes (as f64)
fn bytes_to_gb(size_str: &str) -> f64 {
    let bytes: f64 = size_str.parse().unwrap_or(0.0);
    bytes / (1024.0 * 1024.0 * 1024.0)
}

fn convert_to_message(data: &str) -> String {
    let mut message = String::new();

    // Split the data into lines and remove empty lines
    let lines: Vec<&str> = data
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect();

    // Variables to hold the different sections
    let mut game_packages = Vec::new();
    let mut audio_packages = Vec::new();
    let mut is_game_package = false;
    let mut is_audio_package = false;

    for line in lines {
        if line == "Game Packages:" {
            is_game_package = true;
            is_audio_package = false;
            continue;
        } else if line == "Audio Packages:" {
            is_game_package = false;
            is_audio_package = true;
            continue;
        }

        if is_game_package {
            game_packages.push(line);
        } else if is_audio_package {
            audio_packages.push(line);
        }
    }

    // Process game packages
    let mut i = 0;
    while i < game_packages.len() {
        if game_packages[i].starts_with("[Part") {
            let part_number = game_packages[i]
                .trim_start_matches("[Part ")
                .trim_end_matches("]");
            i += 1;
            let mut url = String::new();
            let mut size = String::new();

            while i < game_packages.len() && !game_packages[i].starts_with("[Part") {
                if game_packages[i].starts_with("[URL] ") {
                    url = game_packages[i].replace("[URL] ", "");
                } else if game_packages[i].starts_with("[Size] ") {
                    size = game_packages[i]
                        .replace("[Size] ", "")
                        .replace("GB", "")
                        .trim()
                        .to_string();
                }
                i += 1;
            }
            message.push_str(&format!("Part {} ({}GB):\n{}\n", part_number, size, url));
        } else {
            i += 1;
        }
    }

    // Add audio package message
    message.push_str("\nYou also need to download an audio pack corresponding to your system's region language.\n");

    // Map language codes to language names
    let language_map = HashMap::from([
        ("zh-cn", "Chinese"),
        ("en-us", "English"),
        ("ko-kr", "Korean"),
        ("ja-jp", "Japanese"),
    ]);

    // Process audio packages
    let mut i = 0;
    while i < audio_packages.len() {
        if audio_packages[i].starts_with("[Language] ") {
            let language_code = audio_packages[i].replace("[Language] ", "");
            let language = language_map
                .get(language_code.as_str())
                .copied()
                .unwrap_or(language_code.as_str());
            i += 1;
            let mut url = String::new();
            let mut size = String::new();

            while i < audio_packages.len() && !audio_packages[i].starts_with("[Language] ") {
                if audio_packages[i].starts_with("[URL] ") {
                    url = audio_packages[i].replace("[URL] ", "");
                } else if audio_packages[i].starts_with("[Size] ") {
                    size = audio_packages[i]
                        .replace("[Size] ", "")
                        .replace("GB", "")
                        .trim()
                        .to_string();
                }
                i += 1;
            }
            // Special handling for Chinese and English
            let language_note = match language {
                "Chinese" => "China",
                "English" => "English - If anything else",
                _ => language,
            };

            message.push_str(&format!("{} ({}GB):\n{}\n", language_note, size, url));
        } else {
            i += 1;
        }
    }

    message
}

