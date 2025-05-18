use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};
use tauri::api::path::app_config_dir;
use tauri::api::path::app_data_dir;
use std::collections::HashMap;

// Estructura de configuración que se puede compartir con el frontend
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OcrCoords {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

// Estructura para manejar las estadísticas seleccionadas
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub token: String,
    pub openai_api_key: String,
    pub ocr_coords: OcrCoords,
    pub server_url: String,
    pub sala_default: String,
    pub hotkey: String,
    pub modo_automatico: bool,
    pub auto_check_interval: i32,
    pub mostrar_stats: bool,
    pub mostrar_analisis: bool,
    pub tema: String,
    pub idioma_ocr: String,
    pub mostrar_dialogo_copia: bool,
    pub stats_seleccionadas: HashMap<String, bool>,
    pub stats_order: Vec<String>,
    pub stats_format: HashMap<String, String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        let mut stats_seleccionadas = HashMap::new();
        let default_stats = vec![
            "vpip", "pfr", "three_bet", "fold_to_3bet_pct", "wtsd", "wsd", 
            "cbet_flop", "cbet_turn", "fold_to_flop_cbet_pct", "fold_to_turn_cbet_pct",
            "limp_pct", "limp_raise_pct", "four_bet_preflop_pct", "fold_to_4bet_pct",
            "probe_bet_turn_pct", "bet_river_pct", "fold_to_river_bet_pct", 
            "overbet_turn_pct", "overbet_river_pct", "wsdwbr_pct", "wwsf", 
            "total_manos", "bb_100", "win_usd"
        ];
        
        // Establecer valores predeterminados para estadísticas
        for stat in default_stats.iter() {
            if ["vpip", "pfr", "three_bet", "fold_to_3bet_pct", "wtsd", "wsd", "cbet_flop", "cbet_turn"].contains(stat) {
                stats_seleccionadas.insert(stat.to_string(), true);
            } else {
                stats_seleccionadas.insert(stat.to_string(), false);
            }
        }
        
        let mut stats_format = HashMap::new();
        stats_format.insert("vpip".to_string(), "VPIP:{value}".to_string());
        stats_format.insert("pfr".to_string(), "PFR:{value}".to_string());
        stats_format.insert("three_bet".to_string(), "3B:{value}".to_string());
        stats_format.insert("fold_to_3bet_pct".to_string(), "F3B:{value}".to_string());
        stats_format.insert("wtsd".to_string(), "WTSD:{value}".to_string());
        stats_format.insert("wsd".to_string(), "WSD:{value}".to_string());
        stats_format.insert("cbet_flop".to_string(), "CF:{value}".to_string());
        stats_format.insert("cbet_turn".to_string(), "CT:{value}".to_string());
        
        AppConfig {
            token: String::new(),
            openai_api_key: String::new(),
            ocr_coords: OcrCoords { x: 95, y: 110, w: 95, h: 22 },
            server_url: "http://localhost:3000".to_string(),
            sala_default: "XPK".to_string(),
            hotkey: "alt+q".to_string(),
            modo_automatico: false,
            auto_check_interval: 30,
            mostrar_stats: true,
            mostrar_analisis: true,
            tema: "dark".to_string(),
            idioma_ocr: "ch".to_string(),
            mostrar_dialogo_copia: false,
            stats_seleccionadas,
            stats_order: default_stats.into_iter().map(|s| s.to_string()).collect(),
            stats_format,
        }
    }
}

// Función para cargar la configuración
pub fn load_config() -> AppConfig {
    // Obtenemos la ruta del directorio de configuración de la app
    let app_config_dir = tauri::api::path::app_config_dir(&tauri::Config::default())
        .expect("No se pudo determinar el directorio de configuración");
    
    let config_path = app_config_dir.join("config.json");
    
    if !config_path.exists() {
        // Si no existe, creamos la configuración por defecto
        let default_config = AppConfig::default();
        save_config(default_config.clone()).ok();
        return default_config;
    }
    
    match fs::read_to_string(&config_path) {
        Ok(content) => {
            match serde_json::from_str::<AppConfig>(&content) {
                Ok(config) => config,
                Err(e) => {
                    println!("Error al deserializar la configuración: {}", e);
                    // Crear copia de seguridad del archivo corrupto
                    let backup_path = app_config_dir.join(format!("config_backup_{}.json", 
                                                                chrono::Local::now().format("%Y%m%d_%H%M%S")));
                    fs::copy(&config_path, &backup_path).ok();
                    
                    // Devolver configuración por defecto
                    let default_config = AppConfig::default();
                    save_config(default_config.clone()).ok();
                    default_config
                }
            }
        },
        Err(e) => {
            println!("Error al leer archivo de configuración: {}", e);
            // Devolver configuración por defecto
            let default_config = AppConfig::default();
            save_config(default_config.clone()).ok();
            default_config
        }
    }
}

// Función para guardar la configuración
pub fn save_config(config: AppConfig) -> Result<(), String> {
    // Obtenemos la ruta del directorio de configuración de la app
    let app_config_dir = app_config_dir(&tauri::Config::default())
        .expect("No se pudo determinar el directorio de configuración");
    
    // Crear directorio si no existe
    if !app_config_dir.exists() {
        fs::create_dir_all(&app_config_dir)
            .map_err(|e| format!("Error al crear directorio de configuración: {}", e))?;
    }
    
    let config_path = app_config_dir.join("config.json");
    
    // Serializar la configuración
    let serialized = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("Error al serializar configuración: {}", e))?;
    
    // Guardar en el archivo
    fs::write(&config_path, serialized)
        .map_err(|e| format!("Error al escribir archivo de configuración: {}", e))?;
    
    Ok(())
}

// Función para restablecer la configuración a valores por defecto
pub fn reset_config() -> Result<(), String> {
    let default_config = AppConfig::default();
    save_config(default_config)
}

// Función para obtener nombres legibles de estadísticas
pub fn get_stat_display_name(stat_key: &str) -> String {
    match stat_key {
        "vpip" => "VPIP".to_string(),
        "pfr" => "PFR".to_string(),
        "three_bet" => "3-Bet".to_string(),
        "fold_to_3bet_pct" => "Fold to 3-Bet".to_string(),
        "wtsd" => "WTSD".to_string(),
        "wsd" => "WSD".to_string(),
        "cbet_flop" => "C-Bet Flop".to_string(),
        "cbet_turn" => "C-Bet Turn".to_string(),
        "fold_to_flop_cbet_pct" => "Fold to Flop C-Bet".to_string(),
        "fold_to_turn_cbet_pct" => "Fold to Turn C-Bet".to_string(),
        "limp_pct" => "Limp %".to_string(),
        "limp_raise_pct" => "Limp-Raise %".to_string(),
        "four_bet_preflop_pct" => "4-Bet Preflop".to_string(),
        "fold_to_4bet_pct" => "Fold to 4-Bet".to_string(),
        "probe_bet_turn_pct" => "Probe Bet Turn".to_string(),
        "bet_river_pct" => "Bet River".to_string(),
        "fold_to_river_bet_pct" => "Fold to River Bet".to_string(),
        "overbet_turn_pct" => "Overbet Turn".to_string(),
        "overbet_river_pct" => "Overbet River".to_string(),
        "wsdwbr_pct" => "WSD with Bet River".to_string(),
        "wwsf" => "WWSF".to_string(),
        "total_manos" => "Total Manos".to_string(),
        "bb_100" => "BB/100".to_string(),
        "win_usd" => "Ganancias USD".to_string(),
        _ => stat_key.to_uppercase(),
    }
}