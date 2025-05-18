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
  useToast,
  Divider,
  HStack,
  VStack,
  Heading,
  Card,
  CardHeader,
  CardBody,
} from '@chakra-ui/react';
import { FaSync, FaSearch, FaDesktop, FaMouse, FaChess, FaCopy, FaPlay, FaStop } from 'react-icons/fa';
import SectionHeader from '../ui/SectionHeader';
import { useMesas } from '../../hooks/useMesas';
import { loadConfig, getWindowUnderCursor, analyzeTable, copyToClipboard, clearNickCache, setupPythonEnvironment, checkOcrAvailable } from '../../services/tauri';

const MesasDetectadas = () => {
  const [selectedMesa, setSelectedMesa] = useState(null);
  const [config, setConfig] = useState(null);
  const [analyzing, setAnalyzing] = useState(false);
  const [analyzeResult, setAnalyzeResult] = useState(null);
  const [autoMode, setAutoMode] = useState(false);
  const [ocrAvailable, setOcrAvailable] = useState(false);
  const toast = useToast();
  
  // Colores para tema claro/oscuro
  const tableBg = useColorModeValue('white', 'gray.800');
  const headerBg = useColorModeValue('gray.50', 'gray.700');
  const borderColor = useColorModeValue('gray.200', 'gray.600');
  const hoverBg = useColorModeValue('blue.50', 'gray.700');
  const resultBg = useColorModeValue('gray.50', 'gray.700');
  const cardBg = useColorModeValue('white', 'gray.800');
  
  // Usamos nuestro hook para manejar las mesas
  const { mesas, loading, error, refreshMesas } = useMesas();
  
  // Verificar OCR al inicio
  useEffect(() => {
    const checkOcr = async () => {
      try {
        const available = await checkOcrAvailable();
        setOcrAvailable(available);
        
        if (!available) {
          // Si OCR no está disponible, intentar configurarlo
          await setupPythonEnvironment();
          // Verificar de nuevo
          const nowAvailable = await checkOcrAvailable();
          setOcrAvailable(nowAvailable);
          
          if (nowAvailable) {
            toast({
              title: "OCR configurado",
              description: "Python y OCR configurados correctamente",
              status: "success",
              duration: 5000,
              isClosable: true,
            });
          } else {
            toast({
              title: "OCR no disponible",
              description: "No se pudo configurar el sistema OCR. Algunas funciones pueden no estar disponibles.",
              status: "warning",
              duration: 8000,
              isClosable: true,
            });
          }
        }
      } catch (err) {
        console.error("Error al verificar OCR:", err);
      }
    };
    
    checkOcr();
  }, [toast]);

  // Cargar la configuración al inicio
  useEffect(() => {
    const fetchConfig = async () => {
      try {
        const appConfig = await loadConfig();
        setConfig(appConfig);
        
        // Si la configuración indica modo automático, activarlo
        if (appConfig.modo_automatico) {
          setAutoMode(true);
        }
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

  // Efecto para modo automático
  useEffect(() => {
    let intervalId = null;
    
    const runAutoMode = async () => {
      if (!autoMode || !config) return;
      
      try {
        // Intentar obtener mesas
        const tables = await refreshMesas();
        
        // Si hay mesas, analizar la primera
        if (tables && tables.length > 0) {
          const firstTable = tables[0];
          // Analizar sin forzar nueva captura para aprovechar caché
          await handleAnalyzeMesaById(firstTable.id, false);
        }
      } catch (err) {
        console.error("Error en modo automático:", err);
      }
    };
    
    if (autoMode) {
      // Ejecutar inmediatamente la primera vez
      runAutoMode();
      
      // Configurar intervalo según la configuración
      const interval = config?.auto_check_interval || 30;
      intervalId = setInterval(runAutoMode, interval * 1000);
      
      toast({
        title: "Modo automático activado",
        description: `Analizando mesas cada ${config?.auto_check_interval || 30} segundos`,
        status: "info",
        duration: 3000,
        isClosable: true,
      });
    }
    
    // Limpiar intervalo al desmontar o cambiar estado
    return () => {
      if (intervalId) clearInterval(intervalId);
    };
  }, [autoMode, config, refreshMesas, toast]);

  // Función para seleccionar una mesa
  const handleSelectMesa = (mesa) => {
    setSelectedMesa(mesa.id === selectedMesa ? null : mesa.id);
  };
  
  // Función para analizar mesa por ID
  const handleAnalyzeMesaById = async (mesaId, forceNew = true) => {
    if (!config) return;
    
    try {
      setAnalyzing(true);
      
      const result = await analyzeTable(mesaId, null, forceNew, config);
      setAnalyzeResult(result);
      
      toast({
        title: "Análisis completado",
        description: "Análisis de jugador completado",
        status: "success",
        duration: 3000,
        isClosable: true,
      });
      
      return result;
    } catch (err) {
      toast({
        title: "Error",
        description: String(err),
        status: "error",
        duration: 5000,
        isClosable: true,
      });
      return null;
    } finally {
      setAnalyzing(false);
    }
  };

  // Función para analizar mesa seleccionada
  const handleAnalyzeMesa = async () => {
    if (!selectedMesa || !config) return;
    await handleAnalyzeMesaById(selectedMesa, true);
  };
  
  // Función para analizar mesa bajo cursor
  const handleAnalyzeCursor = async () => {
    if (!config) return;
    
    try {
      setAnalyzing(true);
      
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
        setAnalyzing(false);
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
      const analysisResult = await analyzeTable(hwnd, null, true, config);
      setAnalyzeResult(analysisResult);
      
      toast({
        title: "Análisis completado",
        description: "Análisis de jugador completado",
        status: "success",
        duration: 5000,
        isClosable: true,
      });
    } catch (err) {
      toast({
        title: "Error",
        description: String(err),
        status: "error",
        duration: 5000,
        isClosable: true,
      });
    } finally {
      setAnalyzing(false);
    }
  };
  
  // Función para limpiar caché
  const handleClearCache = async () => {
    try {
      await clearNickCache();
      toast({
        title: "Caché limpiada",
        description: "La caché de nicks ha sido limpiada correctamente",
        status: "success",
        duration: 3000,
        isClosable: true,
      });
    } catch (err) {
      toast({
        title: "Error",
        description: "No se pudo limpiar la caché",
        status: "error",
        duration: 5000,
        isClosable: true,
      });
    }
  };
  
  // Función para cambiar estado de modo automático
  const toggleAutoMode = () => {
    setAutoMode(!autoMode);
    
    // Guardar preferencia en la configuración
    if (config) {
      const newConfig = { ...config, modo_automatico: !autoMode };
      loadConfig(newConfig).catch(console.error);
    }
  };

  // Función para copiar resultado al portapapeles
  const handleCopyResult = () => {
    if (!analyzeResult) return;
    
    try {
      copyToClipboard(analyzeResult);
      toast({
        title: "Copiado",
        description: "Resultado copiado al portapapeles",
        status: "success",
        duration: 2000,
        isClosable: true,
      });
    } catch (err) {
      toast({
        title: "Error",
        description: "No se pudo copiar al portapapeles",
        status: "error",
        duration: 5000,
        isClosable: true,
      });
    }
  };

  return (
    <Box>
      {/* Estado del OCR */}
      {!ocrAvailable && (
        <Box 
          mb={4} 
          p={3} 
          bg="yellow.100" 
          color="yellow.800" 
          borderRadius="md"
          borderLeft="4px solid" 
          borderColor="yellow.500"
        >
          <Text fontWeight="bold">Advertencia: Sistema OCR no disponible</Text>
          <Text>Algunas funcionalidades podrían no funcionar correctamente. Verifica la instalación de Python y PaddleOCR.</Text>
        </Box>
      )}

      {/* Búsqueda Manual - Similar a la imagen de referencia */}
      <Card mb={5} bg={cardBg} boxShadow="sm">
        <CardHeader bg={headerBg} pb={2}>
          <Heading size="md">Búsqueda Manual</Heading>
        </CardHeader>
        <CardBody>
          <Flex direction="column" gap={3}>
            {/* TODO: Implementar campo de búsqueda manual como en la imagen */}
          </Flex>
        </CardBody>
      </Card>

      {/* Acciones Rápidas - Similar a la imagen de referencia */}
      <Card mb={5} bg={cardBg} boxShadow="sm">
        <CardHeader bg={headerBg} pb={2}>
          <Heading size="md">Acciones Rápidas</Heading>
        </CardHeader>
        <CardBody>
          <VStack align="stretch" spacing={4}>
            <Flex align="center">
              <Text width="150px">Última búsqueda:</Text>
              <HStack>
                <Button size="sm" colorScheme="green" leftIcon={<FaCopy />}>
                  Stats
                </Button>
                <Button size="sm" colorScheme="blue" leftIcon={<FaCopy />}>
                  Análisis
                </Button>
                <Button size="sm" colorScheme="orange" leftIcon={<FaCopy />}>
                  Ambos
                </Button>
              </HStack>
            </Flex>
            
            <Flex align="center">
              <Text width="150px">Configuración:</Text>
              <Button size="sm" colorScheme="gray" leftIcon={<FaSearch />}>
                Seleccionar Stats
              </Button>
            </Flex>
          </VStack>
        </CardBody>
      </Card>

      {/* Opciones de Visualización - Similar a la imagen de referencia */}
      <Card mb={5} bg={cardBg} boxShadow="sm">
        <CardHeader bg={headerBg} pb={2}>
          <Heading size="md">Opciones de Visualización</Heading>
        </CardHeader>
        <CardBody>
          <Flex align="center">
            <Text mr={4}>Incluir en la salida:</Text>
            <HStack spacing={4}>
              {/* Implementar estos como checkboxes reales que afecten a la configuración */}
              <Flex align="center">
                <Box 
                  w="20px" 
                  h="20px" 
                  bg={config?.mostrar_stats ? "blue.500" : "gray.200"} 
                  borderRadius="md" 
                  mr={2}
                />
                <Text>Estadísticas</Text>
              </Flex>
              <Flex align="center">
                <Box 
                  w="20px" 
                  h="20px" 
                  bg={config?.mostrar_analisis ? "blue.500" : "gray.200"} 
                  borderRadius="md" 
                  mr={2}
                />
                <Text>Análisis</Text>
              </Flex>
              <Flex align="center">
                <Box 
                  w="20px" 
                  h="20px" 
                  bg={config?.mostrar_dialogo_copia ? "blue.500" : "gray.200"} 
                  borderRadius="md" 
                  mr={2}
                />
                <Text>Mostrar diálogo de copia</Text>
              </Flex>
            </HStack>
          </Flex>
        </CardBody>
      </Card>

      {/* Estado del Sistema - Similar a la imagen de referencia */}
      <Card mb={5} bg={cardBg} boxShadow="sm">
        <CardHeader bg={headerBg} pb={2}>
          <Heading size="md">Estado del Sistema</Heading>
        </CardHeader>
        <CardBody>
          <Flex justify="space-between" align="center">
            <Flex align="center">
              <Text mr={4}>Estado del modo automático:</Text>
              <Badge 
                colorScheme={autoMode ? "green" : "red"} 
                fontSize="0.9em" 
                px={2} 
                py={1}
                borderRadius="md"
              >
                {autoMode ? "Activo" : "Inactivo"}
              </Badge>
            </Flex>
            <Button
              leftIcon={autoMode ? <FaStop /> : <FaPlay />}
              colorScheme={autoMode ? "red" : "green"}
              size="sm"
              onClick={toggleAutoMode}
              isLoading={analyzing}
            >
              {autoMode ? "Detener" : "Iniciar"} modo automático
            </Button>
          </Flex>
        </CardBody>
      </Card>

      {/* Mesas Detectadas - Tabla principal */}
      <Card bg={cardBg} boxShadow="sm">
        <CardHeader bg={headerBg} pb={2}>
          <Heading size="md">Mesas Detectadas</Heading>
        </CardHeader>
        <CardBody>
          {/* Tabla de mesas */}
          <Box 
            borderWidth="1px" 
            borderRadius="lg" 
            overflow="hidden" 
            bg={tableBg}
            borderColor={borderColor}
            mb={4}
          >
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

          {/* Barra de herramientas de mesas */}
          <Flex>
            <Button 
              leftIcon={<FaSync />} 
              colorScheme="teal" 
              size="sm" 
              onClick={refreshMesas}
              isLoading={loading}
              mr={2}
            >
              Refrescar Mesas
            </Button>
            
            <Button 
              leftIcon={<FaDesktop />} 
              colorScheme="green" 
              size="sm" 
              onClick={handleAnalyzeMesa}
              isDisabled={!selectedMesa}
              isLoading={analyzing}
              mr={2}
            >
              Analizar Mesa Seleccionada
            </Button>
            
            <Button 
              leftIcon={<FaMouse />} 
              colorScheme="blue" 
              size="sm" 
              onClick={handleAnalyzeCursor}
              isLoading={analyzing}
              mr={2}
            >
              Analizar Mesa Bajo Cursor
            </Button>
            
            <Button 
              colorScheme="orange" 
              size="sm" 
              onClick={handleClearCache}
              ml="auto"
            >
              Limpiar Caché
            </Button>
          </Flex>
          
          {/* Mostrar error si existe */}
          {error && (
            <Box mt={4} p={4} bg="red.100" color="red.800" borderRadius="md">
              <Text fontWeight="bold">Error:</Text>
              <Text>{error}</Text>
            </Box>
          )}

          {/* Resultados del análisis */}
          {analyzeResult && (
            <Box 
              mt={4} 
              p={4} 
              bg={resultBg}
              borderRadius="md"
              borderWidth="1px"
              borderColor={borderColor}
            >
              <Flex justify="space-between" align="center" mb={2}>
                <Text fontSize="lg" fontWeight="bold">Resultados del Análisis</Text>
                <Button 
                  leftIcon={<FaCopy />} 
                  colorScheme="purple" 
                  size="sm" 
                  onClick={handleCopyResult}
                >
                  Copiar al Portapapeles
                </Button>
              </Flex>
              <Divider mb={3} />
              <Box 
                whiteSpace="pre-wrap"
                fontFamily="monospace"
                fontSize="sm"
              >
                {analyzeResult}
              </Box>
            </Box>
          )}
        </CardBody>
      </Card>
    </Box>
  );
};

export default MesasDetectadas;