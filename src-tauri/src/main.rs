// src-tauri/src/main.rs
// Prevents additional console window on Windows in release, DO NOT REMOVE!! 
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]  

use tauri::Manager; 
mod window_manager; 
mod auth; 
mod settings; 
mod api;
mod error;

// Comando para obtener las mesas de póker detectadas 
#[tauri::command] 
fn find_poker_tables() -> Vec<(u32, String)> {
    window_manager::find_poker_tables() 
}  

// Comando para analizar una mesa específica  
#[tauri::command] 
fn analyze_table(hwnd: u32, config: settings::AppConfig) -> Result<String, String> {
    window_manager::analyze_table(hwnd, config) 
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

// Comando de ejemplo (usado para compatibilidad)
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            greet,
            find_poker_tables,
            analyze_table,
            get_window_under_cursor,
            save_config,
            load_config,
            login,
            get_app_version,
            clear_nick_cache,
            get_player_stats,
            analyze_stats
        ])
        .setup(|app| {
            // Inicializar componentes en el arranque
            let config = settings::load_config();

            // Configurar app_handle para ser usado en otros módulos si es necesario
            let app_handle = app.handle();
            
            // Otras inicializaciones (logger, etc.)
            
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("Error al ejecutar la aplicación");
}