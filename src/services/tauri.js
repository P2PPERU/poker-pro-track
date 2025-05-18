// src/services/tauri.js
import { invoke } from '@tauri-apps/api';
import { readText, writeText } from '@tauri-apps/api/clipboard';
import { open, save } from '@tauri-apps/api/dialog';

// Funciones para gestión de ventanas de póker
export const findPokerTables = async () => {
  try {
    return await invoke('find_poker_tables');
  } catch (error) {
    console.error('Error al buscar mesas de póker:', error);
    throw new Error(`Error al buscar mesas de póker: ${error}`);
  }
};

export const analyzeTable = async (hwnd, manualNick = null, forceNewCapture = false, config) => {
  try {
    return await invoke('analyze_table', { 
      hwnd, 
      manualNick, 
      forceNewCapture, 
      config 
    });
  } catch (error) {
    console.error('Error al analizar mesa:', error);
    throw new Error(`Error al analizar mesa: ${error}`);
  }
};

export const getWindowUnderCursor = async () => {
  try {
    return await invoke('get_window_under_cursor');
  } catch (error) {
    console.error('Error al obtener ventana bajo el cursor:', error);
    throw new Error(`Error al obtener ventana bajo el cursor: ${error}`);
  }
};

// Funciones para gestión de configuración
export const saveConfig = async (config) => {
  try {
    return await invoke('save_config', { config });
  } catch (error) {
    console.error('Error al guardar configuración:', error);
    throw new Error(`Error al guardar configuración: ${error}`);
  }
};

export const loadConfig = async () => {
  try {
    return await invoke('load_config');
  } catch (error) {
    console.error('Error al cargar configuración:', error);
    throw new Error(`Error al cargar configuración: ${error}`);
  }
};

// Funciones para autenticación
export const loginUser = async (email, password) => {
  try {
    return await invoke('login', { email, password });
  } catch (error) {
    console.error('Error al iniciar sesión:', error);
    throw new Error(`Error al iniciar sesión: ${error}`);
  }
};

// Funciones para análisis y API
export const getPlayerStats = async (nick, sala) => {
  try {
    return await invoke('get_player_stats', { nick, sala });
  } catch (error) {
    console.error('Error al obtener estadísticas del jugador:', error);
    throw new Error(`Error al obtener estadísticas: ${error}`);
  }
};

export const analyzeStats = async (data) => {
  try {
    return await invoke('analyze_stats', { data });
  } catch (error) {
    console.error('Error al analizar estadísticas:', error);
    throw new Error(`Error al analizar estadísticas: ${error}`);
  }
};

// Función para obtener la versión de la aplicación
export const getAppVersion = async () => {
  try {
    return await invoke('get_app_version');
  } catch (error) {
    console.error('Error al obtener versión:', error);
    return '0.0.0';
  }
};

// Funciones para caché
export const clearNickCache = async () => {
  try {
    return await invoke('clear_nick_cache');
  } catch (error) {
    console.error('Error al limpiar caché:', error);
    throw new Error(`Error al limpiar caché: ${error}`);
  }
};

// Funciones de clipboard
export const copyToClipboard = async (text) => {
  try {
    // Primero intentar comando personalizado
    const result = await invoke('copy_to_clipboard', { text });
    if (!result) {
      // Si falla, usar API estándar
      await writeText(text);
    }
    return true;
  } catch (error) {
    console.error('Error al copiar al portapapeles:', error);
    throw new Error(`Error al copiar al portapapeles: ${error}`);
  }
};

export const getFromClipboard = async () => {
  try {
    return await readText();
  } catch (error) {
    console.error('Error al leer del portapapeles:', error);
    return '';
  }
};

// Funciones para OCR
export const checkOcrAvailable = async () => {
  try {
    return await invoke('check_ocr_available');
  } catch (error) {
    console.error('Error al verificar disponibilidad de OCR:', error);
    return false;
  }
};

export const setupPythonEnvironment = async () => {
  try {
    return await invoke('setup_python_environment');
  } catch (error) {
    console.error('Error al configurar entorno Python:', error);
    throw new Error(`Error al configurar Python: ${error}`);
  }
};

// Funciones de diálogo
export const openFileDialog = async (options = {}) => {
  try {
    return await open(options);
  } catch (error) {
    console.error('Error al abrir diálogo:', error);
    return null;
  }
};

export const saveFileDialog = async (options = {}) => {
  try {
    return await save(options);
  } catch (error) {
    console.error('Error al abrir diálogo de guardado:', error);
    return null;
  }
};