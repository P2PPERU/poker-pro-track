// src-tauri/src/main.rs
// Prevents additional console window on Windows in release, DO NOT REMOVE!! 
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]  

use tauri::Manager; 
use tauri::ClipboardManager; // Importar el trait ClipboardManager para acceder a write_text
mod window_manager; 
mod auth; 
mod settings;
mod api;
mod error;
mod ocr_bridge;
mod python_setup;

// Comando para obtener las mesas de póker detectadas 
#[tauri::command] 
fn find_poker_tables() -> Vec<(u32, String)> {
    window_manager::find_poker_tables() 
}  

// Comando para analizar una mesa específica  
#[tauri::command]
async fn analyze_table(hwnd: u32, config: settings::AppConfig, manual_nick: Option<String>, force_new_capture: bool) -> Result<String, String> {
    window_manager::analyze_table(hwnd, config, manual_nick, force_new_capture).await
}

// Comando para obtener la mesa bajo el cursor 
#[tauri::command] 
fn get_window_under_cursor() -> Option<(u32, String)> {
    window_manager::get_window_under_cursor() 
}  

// Comando para guardar configuración 
#[tauri::command] 
fn save_config(config: settings::AppConfig) -> Result<(), String> {
    settings::save_config(config) 
}  

// Comando para cargar configuración 
#[tauri::command] 
fn load_config() -> settings::AppConfig {
    settings::load_config() 
}  

// Comando para iniciar sesión 
#[tauri::command] 
async fn login(email: String, password: String) -> Result<auth::AuthResponse, String> {
    let config = settings::load_config();
    auth::login(email, password, &config.server_url).await 
}  

// Comando para obtener la versión de la aplicación 
#[tauri::command] 
fn get_app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string() 
}

// Comando para limpiar caché de nicks
#[tauri::command]
fn clear_nick_cache() -> bool {
    window_manager::clear_nick_cache()
}

// Comando para obtener estadísticas de jugador
#[tauri::command] 
async fn get_player_stats(nick: String, sala: String) -> Result<api::PlayerStats, String> {
    let config = settings::load_config();
    api::get_player_stats(nick, sala, config.token, config.server_url).await
}

// Comando para analizar estadísticas
#[tauri::command]
async fn analyze_stats(data: api::PlayerStats) -> Result<String, String> {
    let config = settings::load_config();
    api::analyze_stats(data, config.openai_api_key).await
}

// Comando para copiar al portapapeles - versión corregida con ClipboardManager
#[tauri::command]
fn copy_to_clipboard(app_handle: tauri::AppHandle, text: String) -> Result<bool, String> {
    match app_handle.clipboard_manager().write_text(text) {
        Ok(_) => Ok(true),
        Err(e) => Err(format!("Error al copiar al portapapeles: {}", e))
    }
}

// Comando para verificar la disponibilidad de OCR
#[tauri::command]
fn check_ocr_available() -> bool {
    ocr_bridge::check_ocr_availability()
}

// Comando para configurar el entorno Python
#[tauri::command]
fn setup_python_environment() -> Result<bool, String> {
    match python_setup::ensure_python_env() {
        Ok(_) => Ok(true),
        Err(e) => Err(format!("Error al configurar Python: {}", e))
    }
}

fn main() {
    // Configuración inicial
    let _ = python_setup::ensure_python_env();
    
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            find_poker_tables,
            analyze_table,
            get_window_under_cursor,
            save_config,
            load_config,
            login,
            get_app_version,
            clear_nick_cache,
            get_player_stats,
            analyze_stats,
            copy_to_clipboard,
            check_ocr_available,
            setup_python_environment
        ])
        .setup(|_app| { // Agregado guión bajo para indicar que no se utiliza
            // Inicializar componentes en el arranque
            let config = settings::load_config();
            
            // Crear directorios Python si no existen
            let _ = python_setup::ensure_python_env();
            
            // Pre-inicializar OCR en segundo plano
            std::thread::spawn(move || {
                let mut config_map = std::collections::HashMap::new();
                config_map.insert("idioma_ocr".to_string(), config.idioma_ocr.clone());
                
                match ocr_bridge::initialize_ocr(Some(&config_map)) {
                    Ok(_) => println!("OCR inicializado correctamente en segundo plano"),
                    Err(e) => eprintln!("Error al inicializar OCR: {}", e),
                }
            });
            
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("Error al ejecutar la aplicación");
}