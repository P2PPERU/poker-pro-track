// src/components/mesas/MesasDetectadas.jsx
import React, { useState } from 'react';
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
  useDisclosure
} from '@chakra-ui/react';
import { FaSync, FaSearch, FaDesktop, FaMouse, FaChess } from 'react-icons/fa';
import SectionHeader from '../ui/SectionHeader';
import MesasToolbar from './MesasToolbar';

const MesasDetectadas = () => {
  const [mesas, setMesas] = useState([]);
  const [selectedMesa, setSelectedMesa] = useState(null);
  const [loading, setLoading] = useState(false);
  
  // Colores para tema claro/oscuro
  const tableBg = useColorModeValue('white', 'gray.800');
  const headerBg = useColorModeValue('gray.50', 'gray.700');
  const borderColor = useColorModeValue('gray.200', 'gray.600');
  const hoverBg = useColorModeValue('blue.50', 'gray.700');

  // Función para refrescar la lista de mesas (mock por ahora)
  const refreshMesas = () => {
    setLoading(true);
    // Aquí eventualmente invocaremos la función Tauri
    setTimeout(() => {
      setMesas([
        { id: 1, title: 'X-Poker Mesa #1', players: 6, active: true },
        { id: 2, title: 'X-Poker Mesa #2', players: 4, active: true },
      ]);
      setLoading(false);
    }, 1000);
  };

  // Función para seleccionar una mesa
  const handleSelectMesa = (mesa) => {
    setSelectedMesa(mesa.id === selectedMesa ? null : mesa.id);
  };

  // Función para analizar mesa seleccionada (mock por ahora)
  const handleAnalyzeMesa = () => {
    if (!selectedMesa) return;
    
    // Aquí eventualmente invocaremos la función Tauri para analizar
    console.log(`Analizando mesa ID: ${selectedMesa}`);
  };
  
  // Función para analizar mesa bajo cursor (mock por ahora)
  const handleAnalyzeCursor = () => {
    // Aquí eventualmente invocaremos la función Tauri
    console.log('Analizando mesa bajo cursor');
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
    </Box>
  );
};

export default MesasDetectadas;