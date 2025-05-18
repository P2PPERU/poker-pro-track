// src/hooks/useMesas.js
import { useState, useEffect } from 'react';
// Más adelante importaremos: import { invoke } from '@tauri-apps/api';

export function useMesas() {
  const [mesas, setMesas] = useState([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState(null);

  const refreshMesas = async () => {
    try {
      setLoading(true);
      setError(null);
      
      // En el futuro, esto será una llamada real a Tauri:
      // const result = await invoke('find_poker_tables');
      // setMesas(result);
      
      // Por ahora, usamos datos de muestra:
      setTimeout(() => {
        setMesas([
          { id: 1, title: 'X-Poker Mesa #1', players: 6, active: true },
          { id: 2, title: 'X-Poker Mesa #2', players: 4, active: true },
        ]);
        setLoading(false);
      }, 1000);
    } catch (err) {
      setError(err.toString());
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
    refreshMesas
  };
}