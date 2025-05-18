// src/services/tauri.js - Versión para Tauri 1.x
import { invoke } from '@tauri-apps/api';
import { 
  readText as clipboardReadText, 
  writeText as clipboardWriteText 
} from '@tauri-apps/api/clipboard';
import { open as dialogOpen, save as dialogSave } from '@tauri-apps/api/dialog';

// Funciones para gestión de ventanas de póker
export const findPokerTables = async () => {
  try {
    return await invoke('find_poker_tables');
  } catch (error) {
    console.error('Error al buscar mesas de póker:', error);
    return [];
  }
};

export const analyzeTable = async (hwnd, config) => {
  try {
    return await invoke('analyze_table', { hwnd, config });
  } catch (error) {
    console.error('Error al analizar mesa:', error);
    throw error;
  }
};

export const getWindowUnderCursor = async () => {
  try {
    return await invoke('get_window_under_cursor');
  } catch (error) {
    console.error('Error al obtener ventana bajo el cursor:', error);
    return null;
  }
};

// Funciones para gestión de configuración
export const saveConfig = async (config) => {
  try {
    return await invoke('save_config', { config });
  } catch (error) {
    console.error('Error al guardar configuración:', error);
    throw error;
  }
};

export const loadConfig = async () => {
  try {
    return await invoke('load_config');
  } catch (error) {
    console.error('Error al cargar configuración:', error);
    throw error;
  }
};

// Funciones para autenticación
export const loginUser = async (email, password) => {
  try {
    return await invoke('login', { email, password });
  } catch (error) {
    console.error('Error al iniciar sesión:', error);
    throw error;
  }
};

// Funciones para análisis y API
export const getPlayerStats = async (nick, sala, token, serverUrl) => {
  try {
    return await invoke('get_player_stats', { nick, sala, token, serverUrl });
  } catch (error) {
    console.error('Error al obtener estadísticas del jugador:', error);
    throw error;
  }
};

export const analyzeStats = async (data, apiKey) => {
  try {
    return await invoke('analyze_stats', { data, apiKey });
  } catch (error) {
    console.error('Error al analizar estadísticas:', error);
    throw error;
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

// Funciones de clipboard
export const copyToClipboard = async (text) => {
  try {
    await clipboardWriteText(text);
    return true;
  } catch (error) {
    console.error('Error al copiar al portapapeles:', error);
    return false;
  }
};

export const getFromClipboard = async () => {
  try {
    return await clipboardReadText();
  } catch (error) {
    console.error('Error al leer del portapapeles:', error);
    return '';
  }
};

// Funciones de diálogo
export const openFileDialog = async (options = {}) => {
  try {
    return await dialogOpen(options);
  } catch (error) {
    console.error('Error al abrir diálogo:', error);
    return null;
  }
};

export const saveFileDialog = async (options = {}) => {
  try {
    return await dialogSave(options);
  } catch (error) {
    console.error('Error al abrir diálogo de guardado:', error);
    return null;
  }
};