// src/services/profile-detector.js
import { invoke } from '@tauri-apps/api';
import { register, unregister } from '@tauri-apps/api/globalShortcut';
import { getPlayerStats, analyzeStats, copyToClipboard } from './tauri';

/**
 * Servicio para detectar perfiles de jugadores al hacer clic derecho
 * 
 * Este servicio registra un atajo global y también captura clics derechos
 * para analizar automáticamente el perfil de un jugador cuando se activa
 */
class ProfileDetectorService {
  constructor() {
    this.isActive = false;
    this.hotkeyRegistered = false;
    this.config = null;
    this.lastNick = null;
    this.lastStats = null;
    this.lastAnalysis = null;
    this.callbacks = {
      onDetectionStart: null,
      onNickExtracted: null,
      onStatsReceived: null,
      onAnalysisComplete: null,
      onError: null
    };
  }

  /**
   * Inicializa el servicio con la configuración y callbacks
   */
  initialize(config, callbacks = {}) {
    this.config = config;
    
    // Registrar callbacks
    if (callbacks) {
      this.callbacks = { ...this.callbacks, ...callbacks };
    }
    
    // Registrar comando Tauri para capturar clic derecho
    this._registerRightClickCapture();
    
    // Intentar registrar atajo de teclado global (Alt+Q por defecto)
    this._registerHotkey();
    
    console.log("Detector de perfiles inicializado");
    return true;
  }
  
  /**
   * Activa o desactiva el detector
   */
  toggle() {
    this.isActive = !this.isActive;
    
    // Notificar al backend sobre el cambio de estado
    invoke('set_detector_active', { active: this.isActive })
      .catch(err => console.error("Error al cambiar estado del detector:", err));
      
    return this.isActive;
  }
  
  /**
   * Registra la captura de clic derecho en ventanas de póker
   */
  _registerRightClickCapture() {
    // Este método usará un plugin Tauri personalizado para registrar
    // un gancho a nivel de sistema para capturar eventos de clic derecho
    // en ventanas con títulos que coincidan con patrones de póker
    
    invoke('register_right_click_handler')
      .then(() => {
        console.log("Captura de clic derecho registrada");
        
        // Escuchar evento personalizado desde Rust
        window.__TAURI__.event.listen('profile_right_click', (event) => {
          if (!this.isActive) return;
          
          const { hwnd, x, y, title } = event.payload;
          console.log(`Clic derecho detectado en ventana de póker: ${title}`);
          
          // Iniciar el proceso de detección
          this._startDetection(hwnd, { x, y });
        });
      })
      .catch(err => {
        console.error("Error al registrar captura de clic derecho:", err);
        if (this.callbacks.onError) {
          this.callbacks.onError("No se pudo registrar la captura de clic derecho");
        }
      });
  }
  
  /**
   * Registra atajo de teclado global
   */
  async _registerHotkey() {
    if (this.hotkeyRegistered) return;
    
    try {
      const hotkey = this.config?.hotkey || 'Alt+Q';
      await register(hotkey, () => {
        if (!this.isActive) return;
        
        // Obtener ventana bajo el cursor
        invoke('get_window_under_cursor')
          .then(result => {
            if (!result) return;
            
            const [hwnd, title] = result;
            console.log(`Atajo activado sobre ventana: ${title}`);
            
            // Iniciar detección
            this._startDetection(hwnd);
          })
          .catch(err => console.error("Error al obtener ventana bajo cursor:", err));
      });
      
      this.hotkeyRegistered = true;
      console.log(`Atajo de teclado registrado: ${hotkey}`);
    } catch (err) {
      console.error("Error al registrar atajo de teclado:", err);
    }
  }
  
  /**
   * Inicia el proceso de detección en una ventana específica
   */
  async _startDetection(hwnd, clickCoords = null) {
    if (this.callbacks.onDetectionStart) {
      this.callbacks.onDetectionStart();
    }
    
    try {
      // 1. Verificar si es una ventana de perfil de jugador
      const isProfile = await this._verifyProfileWindow(hwnd);
      if (!isProfile) {
        throw new Error("La ventana no parece ser un perfil de jugador");
      }
      
      // 2. Determinar coordenadas óptimas para OCR
      const ocrCoords = await this._calculateOptimalOcrCoords(hwnd, clickCoords);
      
      // 3. Realizar OCR para extraer el nick
      const nickResult = await invoke('extract_nick', { 
        hwnd, 
        coords: ocrCoords 
      });
      
      if (!nickResult || !nickResult.nick) {
        throw new Error("No se pudo extraer el nick del jugador");
      }
      
      const nick = nickResult.nick;
      this.lastNick = nick;
      
      if (this.callbacks.onNickExtracted) {
        this.callbacks.onNickExtracted(nick);
      }
      
      // 4. Obtener estadísticas del jugador
      const stats = await getPlayerStats(nick, this.config.sala_default);
      this.lastStats = stats;
      
      if (this.callbacks.onStatsReceived) {
        this.callbacks.onStatsReceived(stats);
      }
      
      // 5. Generar análisis con IA
      const analysis = await analyzeStats(stats);
      this.lastAnalysis = analysis;
      
      if (this.callbacks.onAnalysisComplete) {
        this.callbacks.onAnalysisComplete({ nick, stats, analysis });
      }
      
      // 6. Copiar resultados según configuración (opcional, automático)
      if (this.config?.auto_copy) {
        this._copyResults(this.config.auto_copy_type || 'both');
      }
      
      return { nick, stats, analysis };
    } catch (err) {
      console.error("Error en detección de perfil:", err);
      
      if (this.callbacks.onError) {
        this.callbacks.onError(err.toString());
      }
      
      return null;
    }
  }
  
  /**
   * Verifica si una ventana es un perfil de jugador
   */
  async _verifyProfileWindow(hwnd) {
    try {
      // Capturar una pequeña porción de la ventana para análisis
      const captureResult = await invoke('capture_window_region', {
        hwnd,
        region: { x: 0, y: 0, w: 400, h: 100 }
      });
      
      // Análisis basado en colores/patrones
      // En la implementación real, esto debería comparar la imagen
      // con un patrón conocido de ventanas de perfil de jugador
      
      // 1. Verificar color de fondo azul (como en la imagen de ejemplo)
      const hasBlueHeader = await invoke('check_color_pattern', {
        imageData: captureResult,
        colorPattern: { r: 0, g: 80, b: 140 },
        threshold: 30,
        minPixels: 1000
      });
      
      // 2. Verificar si contiene avatar (círculo de jugador)
      const hasAvatarCircle = await invoke('detect_avatar_circle', {
        imageData: captureResult
      });
      
      return hasBlueHeader && hasAvatarCircle;
    } catch (err) {
      console.error("Error al verificar ventana de perfil:", err);
      return false;
    }
  }
  
  /**
   * Calcula las coordenadas óptimas para OCR basado en la ventana de perfil
   */
  async _calculateOptimalOcrCoords(hwnd, clickCoords = null) {
    try {
      // Si tenemos coordenadas de clic, podemos usarlas como referencia
      if (clickCoords) {
        // Asumiendo que el nick está en una posición relativa al clic
        return {
          x: clickCoords.x - 50, // Ajustar según el diseño real de la ventana
          y: clickCoords.y - 20,
          w: 200,
          h: 40
        };
      }
      
      // Si no tenemos coordenadas de clic, usar detección basada en patrones
      const windowRect = await invoke('get_window_rect', { hwnd });
      
      // Buscar patrones de color o forma que indiquen la ubicación del nick
      // Esto requiere un algoritmo más sofisticado basado en el análisis de la imagen
      
      // Para la ventana de perfil en la imagen:
      // 1. Buscar el fondo azul del encabezado
      // 2. Encontrar el avatar circular
      // 3. El nick generalmente está justo a la derecha del avatar
      
      // Valores aproximados basados en la imagen de ejemplo
      return {
        x: Math.round(windowRect.width * 0.3),  // ~30% desde la izquierda
        y: Math.round(windowRect.height * 0.2), // ~20% desde arriba
        w: Math.round(windowRect.width * 0.4),  // ~40% de ancho
        h: Math.round(windowRect.height * 0.1)  // ~10% de alto
      };
    } catch (err) {
      console.error("Error al calcular coordenadas OCR:", err);
      // Usar coordenadas por defecto como fallback
      return this.config.ocr_coords;
    }
  }
  
  /**
   * Copia resultados al portapapeles
   */
  _copyResults(type = 'both') {
    if (!this.lastStats || !this.lastNick) return false;
    
    let content = '';
    
    switch(type) {
      case 'stats':
        content = this._formatStats(this.lastStats);
        break;
      case 'analysis':
        content = this.lastAnalysis;
        break;
      case 'both':
      default:
        content = `${this._formatStats(this.lastStats)}\n\n${this.lastAnalysis || ''}`;
        break;
    }
    
    return copyToClipboard(content);
  }
  
  /**
   * Formatea estadísticas para mostrar/copiar
   */
  _formatStats(stats) {
    if (!stats) return '';
    
    // Usar configuración para formatear stats
    const selectedStats = this.config?.stats_seleccionadas || {};
    const statsFormat = this.config?.stats_format || {};
    
    let parts = [];
    
    // Usar solo stats seleccionadas
    Object.keys(selectedStats).forEach(key => {
      if (selectedStats[key] && stats[key]) {
        const format = statsFormat[key] || `${key.toUpperCase()}:{value}`;
        parts.push(format.replace('{value}', stats[key]));
      }
    });
    
    // Si no hay stats seleccionadas, usar un formato predeterminado
    if (parts.length === 0) {
      return `VPIP:${stats.vpip} PFR:${stats.pfr} 3B:${stats.three_bet} F3B:${stats.fold_to_3bet_pct} WTSD:${stats.wtsd} WSD:${stats.wsd} (${stats.total_manos} manos)`;
    }
    
    return parts.join(' ');
  }
  
  /**
   * Limpia recursos al desmontar
   */
  cleanup() {
    // Desregistrar atajo de teclado
    if (this.hotkeyRegistered) {
      const hotkey = this.config?.hotkey || 'Alt+Q';
      unregister(hotkey).catch(console.error);
      this.hotkeyRegistered = false;
    }
    
    // Desregistrar captura de clic derecho
    invoke('unregister_right_click_handler').catch(console.error);
    
    this.isActive = false;
  }
}

// Exportar una única instancia del servicio
export const profileDetector = new ProfileDetectorService();
