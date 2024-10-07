#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

use eframe::egui;
use eframe::egui::{Align, Layout};
use reqwest;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use log::{info, error, debug};

// ----------------------
// Struct Definitions
// ----------------------

// Root API response
#[derive(Deserialize, Serialize, Debug)]
struct ApiResponse {
    retcode: i32,
    message: String,
    data: Data,
}

#[derive(Deserialize, Serialize, Debug)]
struct Data {
    game_packages: Vec<GamePackage>,
}

#[derive(Deserialize, Serialize, Debug)]
struct GamePackage {
    game: Game,
    main: Main,
    pre_download: Option<PreDownload>,
}

#[derive(Deserialize, Serialize, Debug)]
struct Game {
    id: String,
    biz: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct Main {
    major: Option<Major>,
    patches: Vec<Patch>,
    #[serde(default)]
    res_list_url: Option<String>, // Made optional with default
}

#[derive(Deserialize, Serialize, Debug)]
struct PreDownload {
    major: Option<Major>,
    patches: Vec<Patch>,
    #[serde(default)]
    res_list_url: Option<String>, // Made optional with default
}

#[derive(Deserialize, Serialize, Debug)]
struct Major {
    version: String,
    game_pkgs: Vec<Package>,
    audio_pkgs: Vec<AudioPackage>,
}

#[derive(Deserialize, Serialize, Debug)]
struct Package {
    url: String,
    md5: String,
    size: String,
    decompressed_size: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct AudioPackage {
    language: String,
    url: String,
    md5: String,
    size: String,
    decompressed_size: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct Patch {
    version: String,
    game_pkgs: Vec<Package>,
    audio_pkgs: Vec<AudioPackage>,
    #[serde(default)]
    res_list_url: Option<String>, // Made optional with default
}

// ----------------------
// Application State
// ----------------------

struct GenshinApp {
    data: Arc<Mutex<String>>,
    formatted_message: Arc<Mutex<String>>,
    pre_download_main_message: Arc<Mutex<String>>,    // For Pre-download (Main)
    pre_download_patches_message: Arc<Mutex<String>>, // For Pre-download (Patches)
    raw_main_data: Arc<Mutex<String>>,                // Raw JSON for main data
    raw_pre_download_data: Arc<Mutex<String>>,        // Raw JSON for pre-download data
    error_message: Arc<Mutex<String>>,                // Error messages
}

impl Default for GenshinApp {
    fn default() -> Self {
        Self {
            data: Arc::new(Mutex::new(
                "Press 'Fetch Data' to get the latest data.".to_string(),
            )),
            formatted_message: Arc::new(Mutex::new(String::new())),
            pre_download_main_message: Arc::new(Mutex::new(String::new())),      // Initialize empty
            pre_download_patches_message: Arc::new(Mutex::new(String::new())),   // Initialize empty
            raw_main_data: Arc::new(Mutex::new(String::new())),
            raw_pre_download_data: Arc::new(Mutex::new(String::new())),
            error_message: Arc::new(Mutex::new(String::new())),
        }
    }
}

// ----------------------
// eframe Application Implementation
// ----------------------

impl eframe::App for GenshinApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Clone Arc references for thread-safe access
        let data_clone = Arc::clone(&self.data);
        let formatted_message_clone = Arc::clone(&self.formatted_message);
        let pre_download_main_message_clone = Arc::clone(&self.pre_download_main_message);
        let pre_download_patches_message_clone = Arc::clone(&self.pre_download_patches_message);
        let raw_main_data_clone = Arc::clone(&self.raw_main_data);
        let raw_pre_download_data_clone = Arc::clone(&self.raw_pre_download_data);
        let error_message_clone = Arc::clone(&self.error_message);

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
                // Fetch Data Button
                if ui.button("Fetch Data").clicked() {
                    // Clear existing messages and data
                    {
                        let mut message_lock = formatted_message_clone.lock().unwrap();
                        *message_lock = String::new();
                    }
                    {
                        let mut pre_main_lock = pre_download_main_message_clone.lock().unwrap();
                        *pre_main_lock = String::new();
                    }
                    {
                        let mut pre_patches_lock = pre_download_patches_message_clone.lock().unwrap();
                        *pre_patches_lock = String::new();
                    }
                    {
                        let mut error_lock = error_message_clone.lock().unwrap();
                        *error_lock = String::new();
                    }
                    {
                        let mut raw_main_lock = raw_main_data_clone.lock().unwrap();
                        *raw_main_lock = String::new();
                    }
                    {
                        let mut raw_pre_download_lock = raw_pre_download_data_clone.lock().unwrap();
                        *raw_pre_download_lock = String::new();
                    }

                    // Clone Arcs for the thread
                    let data_clone_inner = Arc::clone(&data_clone);
                    let formatted_message_clone_inner = Arc::clone(&formatted_message_clone);
                    let pre_download_main_message_clone_inner = Arc::clone(&pre_download_main_message_clone);
                    let pre_download_patches_message_clone_inner = Arc::clone(&pre_download_patches_message_clone);
                    let raw_main_data_clone_inner = Arc::clone(&raw_main_data_clone);
                    let raw_pre_download_data_clone_inner = Arc::clone(&raw_pre_download_data_clone);
                    let error_message_clone_inner = Arc::clone(&error_message_clone);

                    // Spawn a new thread to fetch data
                    std::thread::spawn(move || {
                        info!("Starting data fetch from API.");
                        match fetch_and_process_data() {
                            Ok((main_data, pre_download_data)) => {
                                info!("Data fetch and processing successful.");

                                // Update main data
                                {
                                    let mut data_lock = data_clone_inner.lock().unwrap();
                                    *data_lock = main_data.clone();
                                }

                                // Convert and update Main Data message
                                let main_message = convert_main_to_message(&main_data);
                                {
                                    let mut formatted_lock = formatted_message_clone_inner.lock().unwrap();
                                    *formatted_lock = main_message;
                                }

                                // Update raw main data
                                {
                                    let mut raw_main_lock = raw_main_data_clone_inner.lock().unwrap();
                                    *raw_main_lock = main_data.clone();
                                }

                                // Update pre_download data if available
                                if let Some(pre_data) = pre_download_data.clone() {
                                    info!("Pre-download data available.");
                                    // Update raw pre-download data
                                    {
                                        let mut raw_pre_download_lock = raw_pre_download_data_clone_inner.lock().unwrap();
                                        *raw_pre_download_lock = pre_data.clone();
                                    }

                                    // Convert pre-download (Main) data
                                    let pre_main_msg = convert_pre_download_main_to_message(&pre_data);
                                    {
                                        let mut pre_main_lock = pre_download_main_message_clone_inner.lock().unwrap();
                                        *pre_main_lock = pre_main_msg;
                                    }

                                    // Convert pre-download (Patches) data
                                    // Extract Current Version from Pre-download (Main)
                                    let current_version = extract_current_version(&pre_data).unwrap_or_else(|| "Unknown".to_string());
                                    let pre_patches_msg = convert_pre_download_patches_to_message(&pre_data, &current_version);
                                    {
                                        let mut pre_patches_lock = pre_download_patches_message_clone_inner.lock().unwrap();
                                        *pre_patches_lock = pre_patches_msg;
                                    }
                                } else {
                                    info!("No pre-download data found.");
                                }
                            }
                            Err(err) => {
                                error!("Error during data fetch: {}", err);
                                let mut error_lock = error_message_clone_inner.lock().unwrap();
                                *error_lock = err;
                            }
                        }
                    });
                }

                // Clear Button
                if ui.button("Clear").clicked() {
                    // Clear all fields
                    {
                        let mut data_lock = data_clone.lock().unwrap();
                        *data_lock = String::new();
                    }
                    {
                        let mut formatted_lock = formatted_message_clone.lock().unwrap();
                        *formatted_lock = String::new();
                    }
                    {
                        let mut pre_main_lock = pre_download_main_message_clone.lock().unwrap();
                        *pre_main_lock = String::new();
                    }
                    {
                        let mut pre_patches_lock = pre_download_patches_message_clone.lock().unwrap();
                        *pre_patches_lock = String::new();
                    }
                    {
                        let mut raw_main_lock = raw_main_data_clone.lock().unwrap();
                        *raw_main_lock = String::new();
                    }
                    {
                        let mut raw_pre_download_lock = raw_pre_download_data_clone.lock().unwrap();
                        *raw_pre_download_lock = String::new();
                    }
                    {
                        let mut error_lock = error_message_clone.lock().unwrap();
                        *error_lock = String::new();
                    }
                }
            });

            ui.separator();

            // Display Error Messages
            let error_message = error_message_clone.lock().unwrap().clone();
            if !error_message.is_empty() {
                ui.colored_label(egui::Color32::RED, &error_message);
                ui.separator();
            }

            // Display the main formatted message with a "Copy" button
            let message = formatted_message_clone.lock().unwrap().clone();
            if !message.is_empty() {
                egui::CollapsingHeader::new("Main Data")
                    .default_open(false) // Set to false to keep collapsed by default
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            if ui.button("Copy").clicked() {
                                ctx.output_mut(|o| o.copied_text = message.clone());
                            }
                        });
                        ui.separator();
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            ui.label(&message);
                        });
                    });
            }

            // Display the pre-download main formatted message with a "Copy" button
            let pre_main_message = pre_download_main_message_clone.lock().unwrap().clone();
            if !pre_main_message.is_empty() {
                egui::CollapsingHeader::new("Pre-download (Main)")
                    .default_open(false)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            if ui.button("Copy").clicked() {
                                ctx.output_mut(|o| o.copied_text = pre_main_message.clone());
                            }
                        });
                        ui.separator();
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            ui.label(&pre_main_message);
                        });
                    });
            }

            // Display the pre-download patches formatted message with a "Copy" button
            let pre_patches_message = pre_download_patches_message_clone.lock().unwrap().clone();
            if !pre_patches_message.is_empty() {
                egui::CollapsingHeader::new("Pre-download (Patches)")
                    .default_open(false)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            if ui.button("Copy").clicked() {
                                ctx.output_mut(|o| o.copied_text = pre_patches_message.clone());
                            }
                        });
                        ui.separator();
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            ui.label(&pre_patches_message);
                        });
                    });
            }

            // Display Raw Main Data for Debugging
            let raw_main_data = raw_main_data_clone.lock().unwrap().clone();
            if !raw_main_data.is_empty() {
                egui::CollapsingHeader::new("Raw Main Data")
                    .default_open(false)
                    .show(ui, |ui| {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            ui.monospace(&raw_main_data);
                        });
                    });
            }

            // Display Raw Pre-download Data for Debugging
            let raw_pre_download_data = raw_pre_download_data_clone.lock().unwrap().clone();
            if !raw_pre_download_data.is_empty() {
                egui::CollapsingHeader::new("Raw Pre-download Data")
                    .default_open(false)
                    .show(ui, |ui| {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            ui.monospace(&raw_pre_download_data);
                        });
                    });
            }
        });
    }
}

// ----------------------
// Main Function
// ----------------------

fn main() -> eframe::Result<()> {
    // Initialize the logger
    env_logger::init();

    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Genshin Package Viewer",
        native_options,
        Box::new(|_cc| Box::new(GenshinApp::default())),
    )
}

// ----------------------
// Helper Functions
// ----------------------

// Function to fetch and process data from the API
fn fetch_and_process_data() -> Result<(String, Option<String>), String> {
    let url = "https://sg-hyp-api.hoyoverse.com/hyp/hyp-connect/api/getGamePackages?game_ids[]=gopR6Cufr3&launcher_id=VYTpXlbWo8";

    info!("Fetching data from URL: {}", url);
    let response = reqwest::blocking::get(url)
        .map_err(|e| {
            error!("Request error: {}", e);
            format!("Request error: {}", e)
        })?
        .text()
        .map_err(|e| {
            error!("Response text error: {}", e);
            format!("Response text error: {}", e)
        })?;

    // Optional: Log the raw response for debugging
    debug!("Raw Response: {}", response);

    // Deserialize the JSON response into ApiResponse struct
    let api_response: ApiResponse = match serde_json::from_str(&response) {
        Ok(res) => {
            info!("Successfully parsed JSON response.");
            res
        },
        Err(e) => {
            error!("JSON parse error: {}", e);
            return Err(format!(
                "JSON parse error: {}\nResponse: {}",
                e, response
            ));
        }
    };

    // Check if API returned an error
    if api_response.retcode != 0 {
        error!("API returned an error: {}", api_response.message);
        return Err(format!("API returned an error: {}", api_response.message));
    }

    // Serialize main data back to JSON string for storage/display
    let main_data = serde_json::to_string(&api_response.data.game_packages)
        .map_err(|e| {
            error!("Serialization error: {}", e);
            format!("Serialization error: {}", e)
        })?;

    // Extract pre_download data if available and serialize it
    let pre_download_data = if let Some(game_package) = api_response.data.game_packages.first() {
        if let Some(pre_download) = &game_package.pre_download {
            Some(
                serde_json::to_string(pre_download)
                    .map_err(|e| {
                        error!("Pre-download Serialization error: {}", e);
                        format!("Pre-download Serialization error: {}", e)
                    })?,
            )
        } else {
            None
        }
    } else {
        None
    };

    Ok((main_data, pre_download_data))
}

// Helper function to extract Current Version from pre_download_data
fn extract_current_version(pre_download_data: &str) -> Option<String> {
    let pre_download: PreDownload = serde_json::from_str(pre_download_data).ok()?;
    pre_download.major.as_ref().map(|m| m.version.clone())
}

// Helper function to convert size in bytes (as string) to gigabytes (as f64)
fn bytes_to_gb(size_str: &str) -> f64 {
    let bytes: f64 = size_str.parse().unwrap_or(0.0);
    bytes / (1024.0 * 1024.0 * 1024.0)
}

// Function to convert main data JSON string to a formatted message
fn convert_main_to_message(data: &str) -> String {
    let game_packages: Vec<GamePackage> = serde_json::from_str(data).unwrap_or_default();
    let mut output = String::new();

    for game_package in game_packages {
        if let Some(major) = game_package.main.major {
            // Game Packages
            output.push_str(&format!("Game Packages (Version {}):\n", major.version));
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
                let language_full = map_language_code(&audio_pkg.language);
                let size_gb = bytes_to_gb(&audio_pkg.size);
                let decompressed_size_gb = bytes_to_gb(&audio_pkg.decompressed_size);
                output.push_str(&format!("[Language] {}\n", language_full));
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

    output
}

// Function to convert pre-download (Main) data JSON string to a formatted message
fn convert_pre_download_main_to_message(pre_download_data: &str) -> String {
    let pre_download: PreDownload = serde_json::from_str(pre_download_data)
        .unwrap_or(PreDownload {
            major: None,
            patches: Vec::new(),
            res_list_url: None,
        });

    if let Some(major) = &pre_download.major {
        let mut output = String::new();

        // Game Packages
        output.push_str(&format!("Pre-download Game Packages (Version {}):\n", major.version));
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
        output.push_str("Pre-download Audio Packages:\n");
        for audio_pkg in &major.audio_pkgs {
            let language_full = map_language_code(&audio_pkg.language);
            let size_gb = bytes_to_gb(&audio_pkg.size);
            let decompressed_size_gb = bytes_to_gb(&audio_pkg.decompressed_size);
            output.push_str(&format!("[Language] {}\n", language_full));
            output.push_str(&format!("[URL] {}\n", audio_pkg.url));
            output.push_str(&format!("[Size] {:.2}GB\n", size_gb));
            output.push_str(&format!(
                "[Decompressed Size] {:.2}GB\n\n",
                decompressed_size_gb
            ));
        }

        output
    } else {
        "No pre-download major version data available.".to_string()
    }
}

// Function to convert pre-download (Patches) data JSON string to a formatted message
fn convert_pre_download_patches_to_message(pre_download_data: &str, current_version: &str) -> String {
    let pre_download: PreDownload = serde_json::from_str(pre_download_data)
        .unwrap_or(PreDownload {
            major: None,
            patches: Vec::new(),
            res_list_url: None,
        });

    if !pre_download.patches.is_empty() {
        let mut output = String::new();
        output.push_str("Pre-download Patches:\n\n");

        for patch in &pre_download.patches {
            // Add heading for each version
            let patch_version_short = patch.version.trim_end_matches(".0").to_string(); // e.g., "5.0.0" -> "5.0"
            output.push_str(&format!("# Version {}\n", patch_version_short));
            // Version line: Previous Version to Current Version
            output.push_str(&format!("Version: {} to {}\n", patch.version, current_version));
            // Game Patch URLs
            for pkg in &patch.game_pkgs {
                let size_gb = bytes_to_gb(&pkg.size);
                let decompressed_size_gb = bytes_to_gb(&pkg.decompressed_size);
                output.push_str(&format!("[Game Patch URL] {}\n", pkg.url));
                output.push_str(&format!("[Size] {:.2}GB\n", size_gb));
                output.push_str(&format!(
                    "[Decompressed Size] {:.2}GB\n\n",
                    decompressed_size_gb
                ));
            }
            // Audio Patch URLs
            for audio_pkg in &patch.audio_pkgs {
                let language_full = map_language_code(&audio_pkg.language);
                let size_gb = bytes_to_gb(&audio_pkg.size);
                let decompressed_size_gb = bytes_to_gb(&audio_pkg.decompressed_size);
                output.push_str(&format!("[Audio Patch Language] {}\n", language_full));
                output.push_str(&format!("[URL] {}\n", audio_pkg.url));
                output.push_str(&format!("[Size] {:.2}GB\n", size_gb));
                output.push_str(&format!(
                    "[Decompressed Size] {:.2}GB\n\n",
                    decompressed_size_gb
                ));
            }
        }

        output
    } else {
        String::new()
    }
}

// ----------------------
// Language Mapping Function
// ----------------------

// Function to map language codes to full names
fn map_language_code(code: &str) -> String {
    match code.to_lowercase().as_str() {
        "zh-cn" => "Chinese".to_string(),
        "en-us" => "English".to_string(),
        "ja-jp" => "Japanese".to_string(),
        "ko-kr" => "Korean".to_string(),
        other => other.to_string(), // Fallback to the original code if not matched
    }
}
