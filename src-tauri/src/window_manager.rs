// src-tauri/src/window_manager.rs
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use crate::settings::AppConfig;
use regex::Regex;
use once_cell::sync::Lazy;

// Esta estructura sería para caché interna
struct NickCache {
    nick: String,
    timestamp: u64,
    img_hash: String,
}

// Caché de nicks (usando Mutex para seguridad en concurrencia)
static NICK_CACHE: Lazy<Mutex<HashMap<u32, NickCache>>> = Lazy::new(|| {
    Mutex::new(HashMap::new())
});

// Función para limpiar la caché
// Nota: No usar #[tauri::command] aquí para evitar conflicto con main.rs
pub fn clear_nick_cache() -> bool {
    match NICK_CACHE.lock() {
        Ok(mut cache) => {
            cache.clear();
            true
        },
        Err(_) => false,
    }
}

// Determina si una ventana es una mesa de póker
pub fn is_poker_table(title: &str) -> bool {
    let regex = Regex::new(r"\d+ *\/ *\d+|\d+bb").unwrap();
    regex.is_match(&title.to_lowercase())
}

// Busca todas las ventanas de mesas de póker activas
pub fn find_poker_tables() -> Vec<(u32, String)> {
    // Simulación de mesas para desarrollo
    #[cfg(debug_assertions)]
    {
        return vec![
            (1, "PokerStars Table - NL Hold'em $1/$2".to_string()),
            (2, "PokerStars Table - NL Hold'em $2/$5".to_string()),
        ];
    }
    
    // Implementación real para producción
    #[cfg(not(debug_assertions))]
    {
        let mut tables = Vec::new();
        
        // Código para detectar ventanas de póker usando win32 API en Windows
        #[cfg(target_os = "windows")]
        unsafe {
            use windows_sys::Win32::UI::WindowsAndMessaging::*;
            use windows_sys::Win32::Foundation::*;
            use std::ptr::null_mut;
            use std::ffi::OsString;
            use std::os::windows::ffi::OsStringExt;
            
            extern "system" fn enum_windows_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
                unsafe {
                    if IsWindowVisible(hwnd) != 0 {
                        let mut title: [u16; 512] = [0; 512];
                        let len = GetWindowTextW(hwnd, title.as_mut_ptr(), title.len() as i32);
                        
                        if len > 0 {
                            let title_os_string = OsString::from_wide(&title[0..len as usize]);
                            let title_str = title_os_string.to_string_lossy().to_string();
                            
                            if is_poker_table(&title_str) {
                                let tables = &mut *(lparam as *mut Vec<(u32, String)>);
                                tables.push((hwnd as u32, title_str));
                            }
                        }
                    }
                    1 // Continuar enumeración
                }
            }
            
            EnumWindows(Some(enum_windows_proc), &mut tables as *mut _ as LPARAM);
            
            // Ordenar por título para consistencia
            tables.sort_by(|a, b| a.1.cmp(&b.1));
        }
        
        // Soporte para macOS
        #[cfg(target_os = "macos")]
        {
            // En una implementación real, usaríamos Objective-C o Swift para detectar ventanas
            // Por ahora devolvemos una simulación
            tables = vec![
                (1, "Simulación macOS - Poker Table #1".to_string()),
                (2, "Simulación macOS - Poker Table #2".to_string()),
            ];
        }
        
        // Soporte para Linux
        #[cfg(target_os = "linux")]
        {
            // Implementación básica para Linux (X11/Wayland)
            // Por ahora devolvemos una simulación
            tables = vec![
                (1, "Simulación Linux - Poker Table #1".to_string()),
                (2, "Simulación Linux - Poker Table #2".to_string()),
            ];
        }
        
        tables
    }
}

// Obtiene el handle de la ventana bajo el cursor
pub fn get_window_under_cursor() -> Option<(u32, String)> {
    // Implementación simulada para desarrollo
    #[cfg(debug_assertions)]
    {
        let tables = find_poker_tables();
        if !tables.is_empty() {
            return Some(tables[0].clone());
        }
        return None;
    }
    
    // Implementación real para producción
    #[cfg(not(debug_assertions))]
    {
        // Código para Windows
        #[cfg(target_os = "windows")]
        unsafe {
            use windows_sys::Win32::UI::WindowsAndMessaging::*;
            use windows_sys::Win32::Foundation::*;
            use std::ffi::OsString;
            use std::os::windows::ffi::OsStringExt;
            
            let mut point = std::mem::zeroed::<POINT>();
            if GetCursorPos(&mut point) == 0 {
                return None;
            }
            
            let hwnd = WindowFromPoint(point);
            if hwnd == 0 {
                return None;
            }
            
            // Obtener título de la ventana
            let mut title: [u16; 512] = [0; 512];
            let len = GetWindowTextW(hwnd, title.as_mut_ptr(), title.len() as i32);
            
            if len > 0 {
                let title_os_string = OsString::from_wide(&title[0..len as usize]);
                let title_str = title_os_string.to_string_lossy().to_string();
                
                if is_poker_table(&title_str) {
                    return Some((hwnd as u32, title_str));
                }
                
                // Verificar ventana padre
                let parent_hwnd = GetParent(hwnd);
                if parent_hwnd != 0 {
                    let mut parent_title: [u16; 512] = [0; 512];
                    let parent_len = GetWindowTextW(parent_hwnd, parent_title.as_mut_ptr(), parent_title.len() as i32);
                    
                    if parent_len > 0 {
                        let parent_title_os_string = OsString::from_wide(&parent_title[0..parent_len as usize]);
                        let parent_title_str = parent_title_os_string.to_string_lossy().to_string();
                        
                        if is_poker_table(&parent_title_str) {
                            return Some((parent_hwnd as u32, parent_title_str));
                        }
                    }
                }
            }
            
            None
        }
        
        // Soporte para macOS
        #[cfg(target_os = "macos")]
        {
            // Implementación simulada para macOS
            let tables = find_poker_tables();
            if !tables.is_empty() {
                return Some(tables[0].clone());
            }
            None
        }
        
        // Soporte para Linux
        #[cfg(target_os = "linux")]
        {
            // Implementación simulada para Linux
            let tables = find_poker_tables();
            if !tables.is_empty() {
                return Some(tables[0].clone());
            }
            None
        }
    }
}

// Analiza una mesa específica
pub fn analyze_table(hwnd: u32, config: AppConfig) -> Result<String, String> {
    // Intentar obtener de la caché primero
    if let Ok(cache) = NICK_CACHE.lock() {
        if let Some(cached_data) = cache.get(&hwnd) {
            // Verificar si la caché es reciente (menos de 5 minutos)
            let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
            if now - cached_data.timestamp < 300 {
                // Usar datos en caché
                return Ok(format!("VPIP:27 PFR:22 3B:7.5 F3B:65 WTSD:31 WSD:54 CB:76/58\n\n🎯 Informe sobre {}:\n\n1️⃣ Estilo de juego: \nJugador TAG moderado que juega muchas manos preflop.\n\n2️⃣ Errores explotables:\n- Fold al 3-bet demasiado alto\n- Llega a showdown con manos débiles\n- Aggressive postflop\n\n3️⃣ Cómo explotarlo:\n3-bet más amplio en posición, value bet thinner en river.", 
                                   cached_data.nick));
            }
        }
    }
    
    // En una implementación real:
    // 1. Capturamos la región de la ventana donde está el nickname
    // 2. Utilizamos OCR para leer el nickname (llamando a Python o una biblioteca)
    // 3. Consultamos la API para obtener estadísticas del jugador
    // 4. Posiblemente usamos GPT para análisis
    
    // Por ahora, devolvemos datos simulados
    let nick = "Jugador123";
    
    // Actualizar caché
    if let Ok(mut cache) = NICK_CACHE.lock() {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let img_hash = generate_image_hash(hwnd, config.ocr_coords.x, config.ocr_coords.y, 
                                          config.ocr_coords.w, config.ocr_coords.h);
        
        cache.insert(hwnd, NickCache {
            nick: nick.to_string(),
            timestamp: now,
            img_hash
        });
    }
    
    // Simulación de llamada a API para obtener estadísticas
    let stats = format!(
        "VPIP:27 PFR:22 3B:7.5 F3B:65 WTSD:31 WSD:54 CB:76/58"
    );
    
    let analysis = format!(
        "🎯 Informe sobre {nick}:\n\n1️⃣ Estilo de juego: \nJugador TAG moderado que juega muchas manos preflop.\n\n2️⃣ Errores explotables:\n- Fold al 3-bet demasiado alto\n- Llega a showdown con manos débiles\n- Aggressive postflop\n\n3️⃣ Cómo explotarlo:\n3-bet más amplio en posición, value bet thinner en river."
    );
    
    Ok(format!("{}\n{}", stats, analysis))
}

// Simulación de hash de imagen (en implementación real usaríamos una biblioteca de procesamiento de imágenes)
fn generate_image_hash(_hwnd: u32, _x: i32, _y: i32, _width: i32, _height: i32) -> String {
    "simulated_hash_123456".to_string()
}

// Captura una región específica de una ventana
fn capture_window_region(_hwnd: u32, _x: i32, _y: i32, _width: i32, _height: i32) -> Vec<u8> {
    // Implementación real: capturar una región de la ventana usando APIs nativas
    // Devolvería bytes de imagen
    Vec::new()
}

// Da foco a una ventana
pub fn focus_window(hwnd: u32) -> Option<(u32, u32)> {
    // Implementación para Windows
    #[cfg(target_os = "windows")]
    unsafe {
        use windows_sys::Win32::UI::WindowsAndMessaging::*;
        use windows_sys::Win32::Foundation::*;
        
        let current_hwnd = GetForegroundWindow();
        if SetForegroundWindow(hwnd as HWND) != 0 {
            // Pequeña pausa para asegurar que la ventana recibe el foco
            std::thread::sleep(std::time::Duration::from_millis(100));
            return Some((hwnd, current_hwnd as u32));
        }
        None
    }
    
    // Implementación mínima para macOS
    #[cfg(target_os = "macos")]
    {
        Some((hwnd, 0))
    }
    
    // Implementación mínima para Linux
    #[cfg(target_os = "linux")]
    {
        Some((hwnd, 0))
    }
    
    // Fallback para otras plataformas
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        Some((hwnd, 0))
    }
}

// Realiza un clic en una posición relativa de una ventana
pub fn click_on_window_point(hwnd: u32, x_offset: i32, y_offset: i32) -> bool {
    // Implementación para Windows
    #[cfg(target_os = "windows")]
    unsafe {
        use windows_sys::Win32::UI::WindowsAndMessaging::*;
        use windows_sys::Win32::Foundation::*;
        
        // Definir manualmente la constante si no está disponible
        const WM_LBUTTON_CONST: u32 = 0x0001;
        
        let result = SendMessageW(
            hwnd as HWND,
            WM_LBUTTONDOWN,
            WM_LBUTTON_CONST as usize,
            ((y_offset << 16) | (x_offset & 0xffff)) as isize
        );
        
        std::thread::sleep(std::time::Duration::from_millis(50));
        
        let result2 = SendMessageW(
            hwnd as HWND,
            WM_LBUTTONUP,
            0,
            ((y_offset << 16) | (x_offset & 0xffff)) as isize
        );
        
        result != 0 && result2 != 0
    }
    
    // Implementación simulada para otras plataformas
    #[cfg(not(target_os = "windows"))]
    {
        true
    }
}