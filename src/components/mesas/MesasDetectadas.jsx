// src/components/mesas/MesasDetectadas.jsx
import React, { useState, useEffect } from 'react';
import {
  Box,
  Table,
  Thead,
  Tbody,
  Tr,
  Th,
  Td,
  Text,
  useColorModeValue,
  Icon,
  Flex,
  Button,
  Badge,
  Tooltip,
  useToast
} from '@chakra-ui/react';
import { FaSync, FaSearch, FaDesktop, FaMouse, FaChess } from 'react-icons/fa';
import SectionHeader from '../ui/SectionHeader';
import MesasToolbar from './MesasToolbar';
import { useMesas } from '../../hooks/useMesas';
import { loadConfig, getWindowUnderCursor, analyzeTable } from '../../services/tauri';

const MesasDetectadas = () => {
  const [selectedMesa, setSelectedMesa] = useState(null);
  const [config, setConfig] = useState(null);
  const toast = useToast();
  
  // Usamos nuestro hook para manejar las mesas
  const { mesas, loading, error, refreshMesas } = useMesas();
  
  // Colores para tema claro/oscuro
  const tableBg = useColorModeValue('white', 'gray.800');
  const headerBg = useColorModeValue('gray.50', 'gray.700');
  const borderColor = useColorModeValue('gray.200', 'gray.600');
  const hoverBg = useColorModeValue('blue.50', 'gray.700');

  // Cargar la configuración al inicio
  useEffect(() => {
    const fetchConfig = async () => {
      try {
        const appConfig = await loadConfig();
        setConfig(appConfig);
      } catch (err) {
        console.error("Error al cargar configuración:", err);
        toast({
          title: "Error",
          description: "No se pudo cargar la configuración",
          status: "error",
          duration: 5000,
          isClosable: true,
        });
      }
    };
    
    fetchConfig();
  }, [toast]);

  // Función para seleccionar una mesa
  const handleSelectMesa = (mesa) => {
    setSelectedMesa(mesa.id === selectedMesa ? null : mesa.id);
  };

  // Función para analizar mesa seleccionada
  const handleAnalyzeMesa = async () => {
    if (!selectedMesa || !config) return;
    
    try {
      toast({
        title: "Analizando",
        description: "Analizando mesa seleccionada...",
        status: "info",
        duration: 2000,
        isClosable: true,
      });
      
      const result = await analyzeTable(selectedMesa, config);
      
      toast({
        title: "Análisis completado",
        description: "Análisis de jugador completado",
        status: "success",
        duration: 5000,
        isClosable: true,
      });
      
      // Mostrar resultados (en una implementación real, podría ser en un modal)
      console.log("Resultado del análisis:", result);
    } catch (err) {
      toast({
        title: "Error",
        description: err.toString(),
        status: "error",
        duration: 5000,
        isClosable: true,
      });
    }
  };
  
  // Función para analizar mesa bajo cursor
  const handleAnalyzeCursor = async () => {
    if (!config) return;
    
    try {
      toast({
        title: "Buscando",
        description: "Buscando mesa bajo el cursor...",
        status: "info",
        duration: 2000,
        isClosable: true,
      });
      
      const result = await getWindowUnderCursor();
      
      if (!result) {
        toast({
          title: "Error",
          description: "No se encontró una mesa de póker bajo el cursor",
          status: "warning",
          duration: 5000,
          isClosable: true,
        });
        return;
      }
      
      const [hwnd, title] = result;
      
      toast({
        title: "Mesa encontrada",
        description: `Mesa encontrada: ${title}`,
        status: "success",
        duration: 2000,
        isClosable: true,
      });
      
      // Analizar la mesa
      const analysisResult = await analyzeTable(hwnd, config);
      
      toast({
        title: "Análisis completado",
        description: "Análisis de jugador completado",
        status: "success",
        duration: 5000,
        isClosable: true,
      });
      
      // Mostrar resultados (en una implementación real, podría ser en un modal)
      console.log("Resultado del análisis:", analysisResult);
    } catch (err) {
      toast({
        title: "Error",
        description: err.toString(),
        status: "error",
        duration: 5000,
        isClosable: true,
      });
    }
  };

  return (
    <Box>
      {/* Encabezado de sección */}
      <SectionHeader 
        title="Mesas Detectadas" 
        icon={FaChess}
        tooltip="Mesas de póker activas detectadas en el sistema"
      />

      {/* Contenedor principal con sombra y bordes redondeados */}
      <Box 
        borderWidth="1px" 
        borderRadius="lg" 
        overflow="hidden" 
        bg={tableBg}
        borderColor={borderColor}
        boxShadow="sm"
        mb={4}
      >
        {/* Tabla de mesas */}
        <Table variant="simple" size="md">
          <Thead bg={headerBg}>
            <Tr>
              <Th width="80px">ID</Th>
              <Th>Título</Th>
            </Tr>
          </Thead>
          <Tbody>
            {mesas.length > 0 ? (
              mesas.map(mesa => (
                <Tr 
                  key={mesa.id}
                  cursor="pointer"
                  onClick={() => handleSelectMesa(mesa)}
                  bg={selectedMesa === mesa.id ? hoverBg : 'transparent'}
                  _hover={{ bg: hoverBg }}
                  transition="background-color 0.2s"
                >
                  <Td>{mesa.id}</Td>
                  <Td>
                    <Flex align="center">
                      <Icon as={FaDesktop} mr={2} color="gray.500" />
                      <Text fontWeight="medium">{mesa.title}</Text>
                      {mesa.active && (
                        <Badge ml={2} colorScheme="green" fontSize="xs">
                          Activa
                        </Badge>
                      )}
                      <Badge ml={2} colorScheme="blue" fontSize="xs">
                        {mesa.players} jugadores
                      </Badge>
                    </Flex>
                  </Td>
                </Tr>
              ))
            ) : (
              <Tr>
                <Td colSpan={2} textAlign="center" py={8}>
                  <Text color="gray.500">No se encontraron mesas de póker activas</Text>
                </Td>
              </Tr>
            )}
          </Tbody>
        </Table>
      </Box>

      {/* Barra de herramientas */}
      <MesasToolbar 
        onRefresh={refreshMesas} 
        onAnalyzeSelected={handleAnalyzeMesa}
        onAnalyzeCursor={handleAnalyzeCursor}
        isSelectedDisabled={!selectedMesa}
        isLoading={loading}
      />
      
      {/* Mostrar error si existe */}
      {error && (
        <Box mt={4} p={4} bg="red.100" color="red.800" borderRadius="md">
          <Text fontWeight="bold">Error:</Text>
          <Text>{error}</Text>
        </Box>
      )}
    </Box>
  );
};

export default MesasDetectadas;
