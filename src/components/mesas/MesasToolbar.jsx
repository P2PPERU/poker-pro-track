// src/components/mesas/MesasToolbar.jsx
import React from 'react';
import { Flex, Button, Icon, ButtonGroup, Tooltip } from '@chakra-ui/react';
import { FaSync, FaSearch, FaDesktop, FaMouse } from 'react-icons/fa';

const MesasToolbar = ({ 
  onRefresh, 
  onAnalyzeSelected, 
  onAnalyzeCursor, 
  isSelectedDisabled, 
  isLoading 
}) => {
  return (
    <Flex justify="space-between">
      <ButtonGroup>
        <Tooltip label="Refrescar lista de mesas">
          <Button 
            leftIcon={<FaSync />} 
            colorScheme="teal" 
            size="sm" 
            onClick={onRefresh}
            isLoading={isLoading}
          >
            Refrescar Mesas
          </Button>
        </Tooltip>
        
        <Tooltip label="Analizar mesa seleccionada">
          <Button 
            leftIcon={<FaDesktop />} 
            colorScheme="green" 
            size="sm" 
            onClick={onAnalyzeSelected}
            isDisabled={isSelectedDisabled}
          >
            Analizar Mesa Seleccionada
          </Button>
        </Tooltip>
      </ButtonGroup>
      
      <Tooltip label="Analizar mesa bajo cursor">
        <Button 
          leftIcon={<FaMouse />} 
          colorScheme="blue" 
          size="sm" 
          onClick={onAnalyzeCursor}
        >
          Analizar Mesa Bajo Cursor
        </Button>
      </Tooltip>
    </Flex>
  );
};

export default MesasToolbar;