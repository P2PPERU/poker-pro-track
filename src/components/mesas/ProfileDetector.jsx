// src/components/mesas/ProfileDetector.jsx
import React, { useState, useEffect, useRef } from 'react';
import {
  Box,
  Button,
  Text,
  Flex,
  Badge,
  HStack,
  VStack,
  Heading,
  Card,
  CardHeader,
  CardBody,
  useToast,
  Switch,
  Icon,
  Tooltip,
  Alert,
  AlertIcon,
  AlertTitle,
  AlertDescription,
  Image,
  ButtonGroup,
  Divider,
  useColorModeValue
} from '@chakra-ui/react';
import { FaSearch, FaUser, FaCopy, FaCog, FaInfoCircle, FaCrosshairs, FaRobot, FaMouse, FaKeyboard } from 'react-icons/fa';

// Importar el nuevo servicio de detección de perfiles
import { profileDetector } from '../../services/profile-detector';
import { loadConfig, saveConfig } from '../../services/tauri';

const ProfileDetector = () => {
  // Estados
  const [isActive, setIsActive] = useState(false);
  const [config, setConfig] = useState(null);
  const [lastDetection, setLastDetection] = useState(null);
  const [lastNick, setLastNick] = useState('');
  const [showPreview, setShowPreview] = useState(false);
  const [capturedNickImage, setCapturedNickImage] = useState(null);
  const [isWorking, setIsWorking] = useState(false);
  const [error, setError] = useState(null);
  
  // Referencias
  const toast = useToast();
  
  // Colores para tema claro/oscuro
  const cardBg = useColorModeValue('white', 'gray.800');
  const headerBg = useColorModeValue('gray.50', 'gray.700');
  const resultBg = useColorModeValue('blue.50', 'blue.900');
  const borderColor = useColorModeValue('gray.200', 'gray.600');
  
  // Cargar configuración al inicio
  useEffect(() => {
    const fetchConfig = async () => {
      try {
        const appConfig = await loadConfig();
        setConfig(appConfig);
      } catch (err) {
        console.error("Error al cargar configuración:", err);
        setError("Error al cargar configuración. Verifica que la aplicación esté correctamente configurada.");
      }
    };
    
    fetchConfig();
  }, []);
  
  // Inicializar detector de perfiles cuando la configuración esté lista
  useEffect(() => {
    if (!config) return;
    
    try {
      // Inicializar con callbacks
      profileDetector.initialize(config, {
        onDetectionStart: () => {
          setIsWorking(true);
          setError(null);
        },
        onNickExtracted: (nick) => {
          setLastNick(nick);
          toast({
            title: "Nick detectado",
            description: `Se ha detectado el nick: ${nick}`,
            status: "info",
            duration: 2000,
            isClosable: true
          });
        },
        onStatsReceived: (stats) => {
          // Podríamos mostrar un preview de las stats aquí
        },
        onAnalysisComplete: (data) => {
          setLastDetection(data);
          setIsWorking(false);
          
          toast({
            title: "Análisis completado",
            description: `Se ha analizado al jugador ${data.nick} correctamente`,
            status: "success",
            duration: 3000,
            isClosable: true
          });
        },
        onError: (errorMsg) => {
          setError(errorMsg);
          setIsWorking(false);
          
          toast({
            title: "Error",
            description: errorMsg,
            status: "error",
            duration: 5000,
            isClosable: true
          });
        }
      });
      
    } catch (err) {
      console.error("Error al inicializar detector de perfiles:", err);
      setError("No se pudo inicializar el detector de perfiles. Reinicia la aplicación.");
    }
  }, [config, toast]);
  
  // Activar/desactivar detector
  const toggleDetector = () => {
    try {
      const newState = profileDetector.toggle();
      setIsActive(newState);
      
      toast({
        title: newState ? "Detector activado" : "Detector desactivado",
        status: newState ? "success" : "info",
        duration: 2000,
        isClosable: true
      });
      
      // Guardar preferencia en configuración
      if (config) {
        const newConfig = {
          ...config,
          detector_active: newState
        };
        saveConfig(newConfig).catch(console.error);
      }
    } catch (err) {
      console.error("Error al cambiar estado del detector:", err);
      setError("No se pudo cambiar el estado del detector");
    }
  };
  
  // Copiar diferentes resultados
  const copyToClipboard = (type) => {
    if (!lastDetection) return;
    
    try {
      let content = '';
      
      switch(type) {
        case 'stats':
          // Formatear stats según configuración
          content = formatStats(lastDetection.stats);
          break;
        case 'analysis':
          content = lastDetection.analysis;
          break;
        case 'both':
          content = `${formatStats(lastDetection.stats)}\n\n${lastDetection.analysis}`;
          break;
        default:
          content = '';
      }
      
      if (content) {
        // Usar el método de copia del detector
        profileDetector._copyResults(type);
        
        toast({
          title: "Copiado",
          description: `${type === 'stats' ? 'Estadísticas' : type === 'analysis' ? 'Análisis' : 'Resultados'} copiados al portapapeles`,
          status: "info",
          duration: 2000,
          isClosable: true
        });
      }
    } catch (err) {
      console.error("Error al copiar:", err);
      setError("No se pudo copiar al portapapeles");
    }
  };
  
  // Formatear estadísticas para visualización
  const formatStats = (stats) => {
    if (!stats) return '';
    return profileDetector._formatStats(stats);
  };
  
  // Limpiar recursos al desmontar
  useEffect(() => {
    return () => {
      try {
        profileDetector.cleanup();
      } catch (err) {
        console.error("Error al limpiar detector:", err);
      }
    };
  }, []);
  
  return (
    <Box>
      {/* Mostrar errores si existen */}
      {error && (
        <Alert status="error" mb={4} borderRadius="md">
          <AlertIcon />
          <AlertTitle>Error</AlertTitle>
          <AlertDescription>{error}</AlertDescription>
        </Alert>
      )}
      
      {/* Card de Estado */}
      <Card mb={5} bg={cardBg} boxShadow="sm">
        <CardHeader bg={headerBg} pb={2}>
          <Flex justify="space-between" align="center">
            <Heading size="md">Detector de Perfiles</Heading>
            <Tooltip label={isActive ? "Desactivar detector" : "Activar detector"}>
              <Switch 
                colorScheme="green" 
                size="lg" 
                isChecked={isActive}
                onChange={toggleDetector}
                isDisabled={isWorking}
              />
            </Tooltip>
          </Flex>
        </CardHeader>
        <CardBody>
          <VStack align="stretch" spacing={4}>
            <Flex align="center" justify="space-between">
              <HStack>
                <Icon 
                  as={FaMouse} 
                  color={isActive ? "green.500" : "gray.400"} 
                  boxSize={5}
                />
                <Text fontWeight="medium">Clic derecho en perfil de jugador</Text>
              </HStack>
              <Badge 
                colorScheme={isActive ? "green" : "gray"} 
                fontSize="0.9em" 
                px={2} 
                py={1}
                borderRadius="md"
              >
                {isActive ? "Activado" : "Desactivado"}
              </Badge>
            </Flex>
            
            <Flex align="center" justify="space-between">
              <HStack>
                <Icon 
                  as={FaKeyboard} 
                  color={isActive ? "blue.500" : "gray.400"} 
                  boxSize={5}
                />
                <Text fontWeight="medium">Atajo de teclado: {config?.hotkey || 'Alt+Q'}</Text>
              </HStack>
              <Badge 
                colorScheme={isActive ? "blue" : "gray"} 
                fontSize="0.9em" 
                px={2} 
                py={1}
                borderRadius="md"
              >
                {isActive ? "Activado" : "Desactivado"}
              </Badge>
            </Flex>
            
            {/* Instrucciones */}
            <Alert status="info" variant="subtle">
              <AlertIcon />
              <Box>
                <AlertTitle>Cómo usar:</AlertTitle>
                <AlertDescription>
                  1. Activa el detector usando el interruptor arriba<br />
                  2. Ve a una mesa de póker y haz clic derecho sobre el perfil de un jugador<br />
                  3. Automáticamente se analizará el jugador y podrás copiar su información
                </AlertDescription>
              </Box>
            </Alert>
          </VStack>
        </CardBody>
      </Card>
      
      {/* Último Análisis */}
      <Card bg={cardBg} boxShadow="sm">
        <CardHeader bg={headerBg} pb={2}>
          <Flex justify="space-between" align="center">
            <Heading size="md">Último Análisis</Heading>
            {lastDetection && (
              <Badge colorScheme="green" fontSize="0.9em" px={2} py={1}>
                {lastDetection.nick}
              </Badge>
            )}
          </Flex>
        </CardHeader>
        <CardBody>
          {isWorking ? (
            <VStack spacing={4} p={4}>
              <Text>Analizando jugador{lastNick ? `: ${lastNick}` : '...'}</Text>
              <Box w="100%" bg="gray.100" h="10px" borderRadius="full" overflow="hidden">
                <Box 
                  bg="blue.500" 
                  h="100%" 
                  w="100%" 
                  borderRadius="full"
                  animation="pulse 1.5s infinite"
                  sx={{
                    "@keyframes pulse": {
                      "0%": { opacity: 0.7 },
                      "50%": { opacity: 1 },
                      "100%": { opacity: 0.7 }
                    }
                  }}
                />
              </Box>
            </VStack>
          ) : lastDetection ? (
            <VStack align="stretch" spacing={4}>
              {/* Información del jugador */}
              <Flex justify="space-between" align="center">
                <HStack>
                  <Icon as={FaUser} color="blue.500" />
                  <Text fontWeight="bold">Jugador:</Text>
                  <Text>{lastDetection.nick}</Text>
                </HStack>
                <Text color="gray.500">Manos: {lastDetection.stats.total_manos}</Text>
              </Flex>
              
              {/* Previsualización de Imagen */}
              {capturedNickImage && showPreview && (
                <Box 
                  p={2} 
                  borderWidth="1px" 
                  borderRadius="md" 
                  borderColor={borderColor}
                  bg="gray.50"
                >
                  <Text fontSize="sm" mb={1}>Captura OCR:</Text>
                  <Image 
                    src={capturedNickImage} 
                    alt="Captura de nick" 
                    maxH="50px" 
                    mx="auto"
                  />
                </Box>
              )}
              
              {/* Estadísticas */}
              <Box 
                p={3} 
                bg={resultBg} 
                borderRadius="md" 
                fontFamily="monospace"
                fontSize="sm"
              >
                {formatStats(lastDetection.stats)}
              </Box>
              
              {/* Análisis */}
              <Box 
                p={3} 
                borderWidth="1px" 
                borderRadius="md" 
                borderColor={borderColor}
                maxH="200px"
                overflow="auto"
                whiteSpace="pre-wrap"
              >
                {lastDetection.analysis}
              </Box>
              
              <Divider />
              
              {/* Botones de acción */}
              <ButtonGroup spacing={3} justifyContent="center">
                <Button 
                  leftIcon={<FaCopy />} 
                  colorScheme="green" 
                  onClick={() => copyToClipboard('stats')}
                >
                  Copiar Stats
                </Button>
                <Button 
                  leftIcon={<FaCopy />} 
                  colorScheme="blue" 
                  onClick={() => copyToClipboard('analysis')}
                >
                  Copiar Análisis
                </Button>
                <Button 
                  leftIcon={<FaCopy />} 
                  colorScheme="purple" 
                  onClick={() => copyToClipboard('both')}
                >
                  Copiar Todo
                </Button>
              </ButtonGroup>
            </VStack>
          ) : (
            <Flex 
              direction="column" 
              align="center" 
              justify="center" 
              p={8} 
              color="gray.500"
            >
              <Icon as={FaSearch} boxSize={12} mb={4} />
              <Text fontSize="lg">Aún no se ha analizado ningún jugador</Text>
              <Text fontSize="sm" mt={2}>
                Activa el detector y haz clic derecho sobre el perfil de un jugador para comenzar
              </Text>
            </Flex>
          )}
        </CardBody>
      </Card>
      
      {/* Opciones de configuración */}
      <Box mt={4} textAlign="right">
        <Button 
          leftIcon={<FaCog />} 
          variant="ghost" 
          size="sm" 
          onClick={() => {/* Abrir configuración */}}
        >
          Configuración avanzada
        </Button>
      </Box>
    </Box>
  );
};

export default ProfileDetector;