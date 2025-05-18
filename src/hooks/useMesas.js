// src/hooks/useMesas.js
import { useState, useEffect, useCallback } from 'react';
import { findPokerTables, analyzeTable } from '../services/tauri';
import { useAuth } from '../context/AuthContext';

export function useMesas() {
  const [mesas, setMesas] = useState([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState(null);
  const { token } = useAuth();
  
  const refreshMesas = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);
      
      // Llamada a Tauri para obtener mesas
      const tables = await findPokerTables();
      
      // Transformar datos para la UI
      const formattedTables = tables.map(([id, title]) => {
        // Extraer información adicional del título
        const isActive = true; // Asumimos activas por defecto
        
        let playerCount = estimatePlayerCount(title);
        let stakeInfo = extractStakeInfo(title);
        let tableType = determineTableType(title);
        
        return {
          id,
          title,
          active: isActive,
          players: playerCount,
          stake: stakeInfo,
          type: tableType
        };
      });
      
      setMesas(formattedTables);
      return formattedTables;
    } catch (err) {
      console.error("Error al obtener mesas:", err);
      setError(err.toString());
      return [];
    } finally {
      setLoading(false);
    }
  }, []);
  
  // Estima el número de jugadores basado en el título de la mesa
  const estimatePlayerCount = (title) => {
    // Buscar patrones como "(6/9)" que indica 6 jugadores de 9 posibles
    const playerMatch = title.match(/\((\d+)\/(\d+)\)/);
    if (playerMatch) {
      return parseInt(playerMatch[1], 10);
    }
    
    // Buscar patrones como "6-max" que indica mesa de 6 jugadores máximo
    const maxMatch = title.match(/(\d+)[- ]max/i);
    if (maxMatch) {
      // Asumimos una mesa llena o casi llena (75% ocupación)
      const maxPlayers = parseInt(maxMatch[1], 10);
      return Math.floor(maxPlayers * 0.75);
    }
    
    // Identificar por tipo de mesa basado en nomenclatura común
    if (title.includes('heads up') || title.includes('HU')) {
      return 2;
    } else if (title.includes('6-max') || title.includes('6max')) {
      return 5; // Asumimos mesa casi llena
    } else if (title.includes('9-max') || title.includes('9max') || title.includes('full ring')) {
      return 7; // Asumimos mesa casi llena
    }
    
    // Valores predeterminados basados en stakes comunes
    if (title.includes('2/$5')) return 6;
    if (title.includes('1/$2')) return 8;
    if (title.includes('0.5/$1')) return 7;
    if (title.includes('0.25/$0.5') || title.includes('25c/50c')) return 8;
    if (title.includes('0.1/$0.25') || title.includes('10c/25c')) return 9;
    
    // Valor por defecto es 6 (mesa de 9 no llena completamente)
    return 6;
  };
  
  // Extrae información de stakes del título
  const extractStakeInfo = (title) => {
    // Patrones comunes de stakes
    const stakesPatterns = [
      // Formatos como $1/$2
      /\$(\d+(?:\.\d+)?)\s*\/\s*\$(\d+(?:\.\d+)?)/i,
      // Formatos como 1/2
      /(\d+(?:\.\d+)?)\s*\/\s*(\d+(?:\.\d+)?)/i,
      // Formatos como $1-$2
      /\$(\d+(?:\.\d+)?)\s*-\s*\$(\d+(?:\.\d+)?)/i,
      // Formatos como 25c/50c
      /(\d+)c\s*\/\s*(\d+)c/i,
      // Formato como 100BB (buy-in en big blinds)
      /(\d+)\s*bb/i
    ];
    
    for (let pattern of stakesPatterns) {
      const match = title.match(pattern);
      if (match) {
        // Si es formato de BB, devolvemos eso
        if (pattern.toString().includes('bb')) {
          return `${match[1]}BB`;
        }
        
        // Para otros formatos, devolvemos small blind / big blind
        const smallBlind = match[1];
        const bigBlind = match[2];
        return `${smallBlind}/${bigBlind}`;
      }
    }
    
    return 'Unknown';
  };
  
  // Determina el tipo de mesa basado en el título
  const determineTableType = (title) => {
    const titleLower = title.toLowerCase();
    
    if (titleLower.includes('tournament') || titleLower.includes('tourney') || titleLower.includes('mtt')) {
      return 'Tournament';
    } else if (titleLower.includes('sit') && titleLower.includes('go') || titleLower.includes('sng')) {
      return 'Sit & Go';
    } else if (titleLower.includes('cash') || titleLower.includes('ring')) {
      return 'Cash Game';
    } else if (titleLower.includes('zoom') || titleLower.includes('fast') || titleLower.includes('rush')) {
      return 'Fast-Fold';
    } else if (titleLower.includes('spin') && titleLower.includes('go')) {
      return 'Spin & Go';
    }
    
    // Detección por sala
    if (titleLower.includes('pokerstars')) {
      return titleLower.includes('zoom') ? 'Fast-Fold' : 'Cash Game';
    } else if (titleLower.includes('ggpoker')) {
      return titleLower.includes('rush') ? 'Fast-Fold' : 'Cash Game';
    } else if (titleLower.includes('888poker')) {
      return titleLower.includes('snap') ? 'Fast-Fold' : 'Cash Game';
    } else if (titleLower.includes('partypoker')) {
      return titleLower.includes('fast forward') ? 'Fast-Fold' : 'Cash Game';
    }
    
    // Valor predeterminado
    return 'Cash Game';
  };
  
  // Analizar mesa seleccionada
  const analyzeMesa = async (mesaId, config) => {
    try {
      setLoading(true);
      const result = await analyzeTable(mesaId, null, true, config);
      return result;
    } catch (err) {
      setError(err.toString());
      throw err;
    } finally {
      setLoading(false);
    }
  };

  // Cargar mesas al iniciar
  useEffect(() => {
    refreshMesas();
  }, [refreshMesas]);

  return {
    mesas,
    loading,
    error,
    refreshMesas,
    analyzeMesa
  };
}