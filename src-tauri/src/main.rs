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
mod right_click_detector;  // Nuevo módulo

use std::collections::HashMap;
use std::sync::Mutex;
use once_cell::sync::Lazy;
use image::{RgbaImage, ImageOutputFormat};
use base64;
use right_click_detector::RightClickDetector;

// Variable global para almacenar el detector
static RIGHT_CLICK_DETECTOR: Lazy<Mutex<Option<RightClickDetector>>> = 
    Lazy::new(|| Mutex::new(None));

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

// NUEVOS COMANDOS

#[tauri::command]
async fn extract_nick(hwnd: u32, coords: HashMap<String, i32>) -> Result<serde_json::Value, String> {
    // Este comando extrae el nick de un jugador de póker de una ventana específica
    // optimizado para ventanas de perfil como la que se muestra en la captura
    
    // Convertir las coordenadas a formato esperado por ocr_bridge
    match ocr_bridge::capture_and_read_nick(hwnd, coords) {
        Ok(result) => {
            // Procesar y limpiar el nick
            let nick = result.nick.trim();
            
            // Intentar limpiar el nick (eliminar posibles caracteres no deseados)
            let clean_nick = nick.trim_start_matches("ID:").trim();
            
            // Registrar detección en log
            println!("Nick extraído: '{}' con confianza: {}", clean_nick, result.confidence);
            
            // Devolver resultado como JSON
            Ok(serde_json::json!({
                "nick": clean_nick,
                "confidence": result.confidence,
                "image_hash": result.image_hash
            }))
        },
        Err(e) => Err(format!("Error al extraer nick: {}", e))
    }
}

#[tauri::command]
fn capture_window_region(hwnd: u32, region: HashMap<String, i32>) -> Result<String, String> {
    // Extraer coordenadas
    let x = region.get("x").cloned().unwrap_or(0);
    let y = region.get("y").cloned().unwrap_or(0);
    let w = region.get("w").cloned().unwrap_or(100);
    let h = region.get("h").cloned().unwrap_or(100);
    
    #[cfg(target_os = "windows")]
    unsafe {
        use windows_sys::Win32::UI::WindowsAndMessaging::*;
        use windows_sys::Win32::Foundation::*;
        use windows_sys::Win32::Graphics::Gdi::*;
        
        let hwnd_native = hwnd as HWND;
        
        // Verificar que la ventana existe
        if IsWindow(hwnd_native) == 0 {
            return Err("La ventana especificada no existe".to_string());
        }
        
        // Capturar la región usando GDI
        let hwnd_dc = GetWindowDC(hwnd_native);
        if hwnd_dc == 0 {
            return Err("No se pudo obtener DC de la ventana".to_string());
        }
        
        let memory_dc = CreateCompatibleDC(hwnd_dc);
        if memory_dc == 0 {
            ReleaseDC(hwnd_native, hwnd_dc);
            return Err("No se pudo crear DC compatible".to_string());
        }
        
        let bitmap = CreateCompatibleBitmap(hwnd_dc, w, h);
        if bitmap == 0 {
            DeleteDC(memory_dc);
            ReleaseDC(hwnd_native, hwnd_dc);
            return Err("No se pudo crear bitmap compatible".to_string());
        }
        
        let old_bitmap = SelectObject(memory_dc, bitmap as isize);
        
        // Copiar pixels de la ventana al bitmap
        let success = BitBlt(memory_dc, 0, 0, w, h, hwnd_dc, x, y, SRCCOPY);
        
        // Liberar recursos GDI
        SelectObject(memory_dc, old_bitmap);
        DeleteDC(memory_dc);
        ReleaseDC(hwnd_native, hwnd_dc);
        
        if success == 0 {
            DeleteObject(bitmap as isize);
            return Err("Error al copiar pixels".to_string());
        }
        
        // Convertir bitmap a RGBA bytes
        let mut buffer = Vec::new();
        buffer.resize((w * h * 4) as usize, 0);
        
        // Obtener información del bitmap
        let mut bitmap_info: BITMAPINFO = std::mem::zeroed();
        bitmap_info.bmiHeader.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
        bitmap_info.bmiHeader.biWidth = w;
        bitmap_info.bmiHeader.biHeight = -h; // Negativo para top-down
        bitmap_info.bmiHeader.biPlanes = 1;
        bitmap_info.bmiHeader.biBitCount = 32;
        bitmap_info.bmiHeader.biCompression = BI_RGB as u32;
        
        let screen_dc = GetDC(0);
        let result = GetDIBits(
            screen_dc,
            bitmap,
            0,
            h as u32,
            buffer.as_mut_ptr() as *mut _,
            &mut bitmap_info,
            DIB_RGB_COLORS
        );
        
        ReleaseDC(0, screen_dc);
        DeleteObject(bitmap as isize);
        
        if result == 0 {
            return Err("Error al obtener datos del bitmap".to_string());
        }
        
        // Convertir RGBA a PNG y codificar en base64
        let img = RgbaImage::from_raw(w as u32, h as u32, buffer).ok_or("Error al crear imagen RGBA")?;
        
        let mut png_data = Vec::new();
        img.write_to(&mut std::io::Cursor::new(&mut png_data), ImageOutputFormat::Png).map_err(|e| e.to_string())?;
        
        // Codificar en base64
        let base64_data = base64::encode(&png_data);
        
        Ok(base64_data)
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        Err("Captura de ventana solo disponible en Windows".to_string())
    }
}

#[tauri::command]
fn register_right_click_handler(app_handle: tauri::AppHandle) -> Result<(), String> {
    match RIGHT_CLICK_DETECTOR.lock() {
        Ok(mut detector_guard) => {
            // Si ya existe un detector, verificar si está activo
            if let Some(detector) = detector_guard.as_mut() {
                // Ya existe, solo activarlo
                detector.set_active(true)?;
                return Ok(());
            }
            
            // Crear nuevo detector
            let mut new_detector = RightClickDetector::new();
            new_detector.set_active(true)?;
            new_detector.start(app_handle)?;
            
            // Guardar en variable global
            *detector_guard = Some(new_detector);
            
            Ok(())
        },
        Err(_) => Err("No se pudo acceder al detector".to_string())
    }
}

#[tauri::command]
fn unregister_right_click_handler() -> Result<(), String> {
    match RIGHT_CLICK_DETECTOR.lock() {
        Ok(mut detector_guard) => {
            if let Some(detector) = detector_guard.as_mut() {
                // Desactivar el detector
                detector.set_active(false)?;
            }
            
            Ok(())
        },
        Err(_) => Err("No se pudo acceder al detector".to_string())
    }
}

#[tauri::command]
fn set_detector_active(active: bool) -> Result<(), String> {
    match RIGHT_CLICK_DETECTOR.lock() {
        Ok(mut detector_guard) => {
            if let Some(detector) = detector_guard.as_mut() {
                detector.set_active(active)?;
                Ok(())
            } else {
                Err("Detector no inicializado".to_string())
            }
        },
        Err(_) => Err("No se pudo acceder al detector".to_string())
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
            setup_python_environment,
            // Nuevos comandos
            extract_nick,
            capture_window_region,
            register_right_click_handler,
            unregister_right_click_handler,
            set_detector_active
        ])
        .setup(|app| { // Agregado guión bajo para indicar que no se utiliza
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