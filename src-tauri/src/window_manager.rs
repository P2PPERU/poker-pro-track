// src-tauri/src/window_manager.rs
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use crate::settings::AppConfig;
use crate::ocr_bridge::{initialize_ocr, capture_and_read_nick}; // Eliminado OcrResult no utilizado
use crate::error::AppError;
use regex::Regex;
use once_cell::sync::Lazy;

// Estructura para caché interna
struct NickCache {
    nick: String,
    timestamp: u64,
    img_hash: String,
    confidence: f32,
}

// Caché de nicks (usando Mutex para seguridad en concurrencia)
static NICK_CACHE: Lazy<Mutex<HashMap<u32, NickCache>>> = Lazy::new(|| {
    Mutex::new(HashMap::new())
});

// Función para limpiar la caché
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
    let patterns = [
        r"\d+ *\/ *\d+",     // Formato "1/2"
        r"\d+bb",            // Formato "100bb"
        r"NL\s*Hold'?em",    // Juego NL Hold'em 
        r"pot\s*limit",      // Pot Limit
        r"poker\s*table",    // "Poker Table" en el título
        r"table\s*\d+",      // "Table 123"
        r"PokerStars",       // Salas comunes
        r"GGPoker",
        r"888poker",
        r"PartyPoker",
        r"Winamax",
        r"iPoker"
    ];
    
    let title_lower = title.to_lowercase();
    patterns.iter().any(|&pattern| {
        Regex::new(pattern).map(|re| re.is_match(&title_lower)).unwrap_or(false)
    })
}

// Busca todas las ventanas de mesas de póker activas
pub fn find_poker_tables() -> Vec<(u32, String)> {
    // Implementación simulada para desarrollo
    #[cfg(debug_assertions)]
    {
        return vec![
            (1, "PokerStars Table - NL Hold'em $1/$2".to_string()),
            (2, "PokerStars Table - NL Hold'em $2/$5".to_string()),
            (3, "GGPoker - $5/$10 No-Limit Hold'em".to_string()),
        ];
    }
    
    // Implementación real para producción en Windows
    #[cfg(all(not(debug_assertions), target_os = "windows"))]
    unsafe {
        use windows_sys::Win32::UI::WindowsAndMessaging::*;
        use windows_sys::Win32::Foundation::*;
        use std::ptr::null_mut;
        use std::ffi::OsString;
        use std::os::windows::ffi::OsStringExt;
        
        let mut tables = Vec::new();
        
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
        tables.sort_by(|a: &(u32, String), b: &(u32, String)| a.1.cmp(&b.1));
        
        tables
    }
    
    // Implementación para otras plataformas
    #[cfg(all(not(debug_assertions), not(target_os = "windows")))]
    {
        vec![
            (1, "Simulación - Poker Table #1".to_string()),
            (2, "Simulación - Poker Table #2".to_string()),
        ]
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
    
    // Implementación real para producción en Windows
    #[cfg(all(not(debug_assertions), target_os = "windows"))]
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
    
    // Implementación para otras plataformas
    #[cfg(all(not(debug_assertions), not(target_os = "windows")))]
    {
        let tables = find_poker_tables();
        if !tables.is_empty() {
            return Some(tables[0].clone());
        }
        None
    }
}

// Analiza una mesa específica
pub async fn analyze_table(hwnd: u32, config: AppConfig, manual_nick: Option<String>, force_new_capture: bool) -> Result<String, String> {
    // Obtener nick del jugador
    let nick = if let Some(nick_str) = manual_nick {
        nick_str
    } else {
        // Intentar obtener de la caché primero
        if let Ok(cache) = NICK_CACHE.lock() {
            if let Some(cached_data) = cache.get(&hwnd) {
                // Verificar si la caché es reciente (menos de 5 minutos)
                let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
                if !force_new_capture && now - cached_data.timestamp < 300 {
                    // Usar datos en caché
                    cached_data.nick.clone()
                } else {
                    // Caché expirada, usar OCR
                    drop(cache); // Liberar mutex antes de OCR
                    detect_nick(hwnd, &config).map_err(|e| e.to_string())?
                }
            } else {
                // No hay caché, usar OCR
                drop(cache); // Liberar mutex antes de OCR
                detect_nick(hwnd, &config).map_err(|e| e.to_string())?
            }
        } else {
            // Error al obtener mutex, usar OCR
            detect_nick(hwnd, &config).map_err(|e| e.to_string())?
        }
    };
    
    // Obtener estadísticas del jugador
    let stats_result = crate::api::get_player_stats(
        nick.clone(), 
        config.sala_default.clone(), 
        config.token.clone(), 
        config.server_url.clone()
    ).await;
    
    let stats = match stats_result {
        Ok(stats_data) => stats_data,
        Err(e) => return Err(format!("Error al obtener estadísticas: {}", e)),
    };
    
    // Formatear estadísticas según preferencias
    let mut response = String::new();
    
    // Incluir stats si están habilitadas
    if config.mostrar_stats {
        let stats_text = format_stats(&stats, &config);
        response.push_str(&stats_text);
        response.push_str("\n\n");
    }
    
    // Incluir análisis si está habilitado
    if config.mostrar_analisis && !config.openai_api_key.is_empty() {
        match crate::api::analyze_stats(stats.clone(), config.openai_api_key.clone()).await {
            Ok(analysis) => {
                response.push_str(&analysis);
            },
            Err(e) => {
                response.push_str(&format!("Error en análisis: {}", e));
            }
        }
    }
    
    // Guardar en historial
    // TODO: Implementar guardar en historial
    
    Ok(response)
}

// Función para detectar nick usando OCR
fn detect_nick(hwnd: u32, config: &AppConfig) -> Result<String, AppError> {
    // Mapear las coordenadas para OCR
    let mut coords = HashMap::new();
    coords.insert("x".to_string(), config.ocr_coords.x);
    coords.insert("y".to_string(), config.ocr_coords.y);
    coords.insert("w".to_string(), config.ocr_coords.w);
    coords.insert("h".to_string(), config.ocr_coords.h);
    
    // Variable para controlar inicialización única
    static OCR_INITIALIZED: Lazy<Mutex<bool>> = Lazy::new(|| Mutex::new(false));
    
    // Inicializar OCR si es necesario
    if let Ok(mut initialized) = OCR_INITIALIZED.lock() {
        if !*initialized {
            let mut config_map = HashMap::new();
            config_map.insert("idioma_ocr".to_string(), config.idioma_ocr.clone());
            
            match initialize_ocr(Some(&config_map)) {
                Ok(_) => *initialized = true,
                Err(e) => return Err(AppError::Api(format!("Error al inicializar OCR: {}", e))),
            }
        }
    }
    
    // Activar ventana
    focus_window(hwnd);
    
    // Llamar a la función de OCR
    let ocr_result = capture_and_read_nick(hwnd, coords)?;
    
    // Guardar en caché
    if let Ok(mut cache) = NICK_CACHE.lock() {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        cache.insert(hwnd, NickCache {
            nick: ocr_result.nick.clone(),
            timestamp: now,
            img_hash: ocr_result.image_hash,
            confidence: ocr_result.confidence,
        });
    }
    
    // Si el nick parece vacío o es un error, devolver error descriptivo
    if ocr_result.nick.is_empty() || ocr_result.nick == "Error" || ocr_result.nick == "ErrorOCR" || ocr_result.nick == "NoText" {
        return Err(AppError::Api("No se pudo detectar nick en la imagen".to_string()));
    }
    
    Ok(ocr_result.nick)
}

// Formatea estadísticas según la configuración del usuario
fn format_stats(stats: &crate::api::PlayerStats, config: &AppConfig) -> String {
    let selected_stats = &config.stats_seleccionadas;
    let stats_order = &config.stats_order;
    let stats_format = &config.stats_format;
    
    // Filtrar solo stats seleccionadas
    let filtered_stats: Vec<_> = stats_order.iter()
        .filter(|&key| selected_stats.get(key).unwrap_or(&false) == &true)
        .collect();
    
    // Si no hay stats seleccionadas, usar conjunto básico
    if filtered_stats.is_empty() {
        return format!(
            "VPIP:{} PFR:{} 3B:{} F3B:{} WTSD:{} WSD:{} CB:{}/{}",
            stats.vpip, stats.pfr, stats.three_bet, stats.fold_to_3bet_pct,
            stats.wtsd, stats.wsd, stats.cbet_flop, stats.cbet_turn
        );
    }
    
    // Construir partes del resumen
    let mut stats_parts = Vec::new();
    
    for stat_key in filtered_stats {
        if let Some(format_str) = stats_format.get(stat_key) {
            let value: &str = match stat_key.as_str() {
                "vpip" => &stats.vpip,
                "pfr" => &stats.pfr,
                "three_bet" => &stats.three_bet,
                "fold_to_3bet_pct" => &stats.fold_to_3bet_pct,
                "wtsd" => &stats.wtsd,
                "wsd" => &stats.wsd,
                "cbet_flop" => &stats.cbet_flop,
                "cbet_turn" => &stats.cbet_turn,
                "fold_to_flop_cbet_pct" => &stats.fold_to_flop_cbet_pct,
                "fold_to_turn_cbet_pct" => &stats.fold_to_turn_cbet_pct,
                "wwsf" => &stats.wwsf,
                "total_manos" => &stats.total_manos,
                "bb_100" => &stats.bb_100,
                "win_usd" => &stats.win_usd,
                "limp_pct" => stats.limp_pct.as_deref().unwrap_or("0"),
                "limp_raise_pct" => stats.limp_raise_pct.as_deref().unwrap_or("0"),
                "four_bet_preflop_pct" => stats.four_bet_preflop_pct.as_deref().unwrap_or("0"),
                "fold_to_4bet_pct" => stats.fold_to_4bet_pct.as_deref().unwrap_or("0"),
                "probe_bet_turn_pct" => stats.probe_bet_turn_pct.as_deref().unwrap_or("0"),
                "bet_river_pct" => stats.bet_river_pct.as_deref().unwrap_or("0"),
                "fold_to_river_bet_pct" => stats.fold_to_river_bet_pct.as_deref().unwrap_or("0"),
                "overbet_turn_pct" => stats.overbet_turn_pct.as_deref().unwrap_or("0"),
                "overbet_river_pct" => stats.overbet_river_pct.as_deref().unwrap_or("0"),
                "wsdwbr_pct" => stats.wsdwbr_pct.as_deref().unwrap_or("0"),
                _ => continue,
            };

            let formatted = format_str.replace("{value}", value);
            stats_parts.push(formatted);
        }
    }
    
    stats_parts.join(" ")
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
    
    // Implementación mínima para otras plataformas
    #[cfg(not(target_os = "windows"))]
    {
        Some((hwnd, 0))
    }
}

// Captura una región específica de una ventana
fn capture_window_region(_hwnd: u32, _region: (i32, i32, i32, i32)) -> Vec<u8> {
    // Implementación real: capturar una región de la ventana usando APIs nativas
    // En implementación real, usaríamos el puente OCR que ya maneja esto
    Vec::new()
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