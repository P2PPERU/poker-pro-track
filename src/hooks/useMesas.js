// src/hooks/useMesas.js
import { useState, useEffect } from 'react';
import { findPokerTables, analyzeTable } from '../services/tauri';
import { useAuth } from '../context/AuthContext';

export function useMesas() {
  const [mesas, setMesas] = useState([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState(null);
  const { token } = useAuth();
  
  const refreshMesas = async () => {
    try {
      setLoading(true);
      setError(null);
      
      // Llamada real a Tauri
      const tables = await findPokerTables();
      
      const formattedTables = tables.map(([id, title]) => ({
        id,
        title,
        active: true,
        players: estimatePlayerCount(title)
      }));
      
      setMesas(formattedTables);
    } catch (err) {
      console.error("Error al obtener mesas:", err);
      setError(err.toString());
    } finally {
      setLoading(false);
    }
  };
  
  // Estima el número de jugadores basado en el título de la mesa (simulación)
  const estimatePlayerCount = (title) => {
    // Extraer información de la cantidad de jugadores, si está disponible en el título
    // Por ejemplo: "9-max Table (6/9)" indicaría 6 jugadores
    const match = title.match(/\((\d+)\/(\d+)\)/);
    if (match) {
      return parseInt(match[1], 10);
    }
    
    // Valores por defecto para demostración
    if (title.includes('2/$5')) return 7;
    if (title.includes('1/$2')) return 9;
    
    // Valor por defecto
    return Math.floor(Math.random() * 6) + 4; // entre 4 y 9 jugadores
  };
  
  // Analizar mesa seleccionada
  const analyzeMesa = async (hwnd, config) => {
    try {
      setLoading(true);
      const result = await analyzeTable(hwnd, config);
      return result;
    } catch (err) {
      setError(err.toString());
      throw err;
    } finally {
      setLoading(false);
    }
  };

  // Cargamos las mesas al iniciar
  useEffect(() => {
    refreshMesas();
  }, []);

  return {
    mesas,
    loading,
    error,
    refreshMesas,
    analyzeMesa
  };
}