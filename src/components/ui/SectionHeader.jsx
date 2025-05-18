// src/components/ui/SectionHeader.jsx
import React from 'react';
import { Flex, Heading, Icon, Tooltip } from '@chakra-ui/react';
import { FaInfoCircle } from 'react-icons/fa';

const SectionHeader = ({ title, icon, tooltip }) => {
  return (
    <Flex 
      align="center" 
      mb={4} 
      bg="transparent"
    >
      {icon && <Icon as={icon} boxSize={5} color="blue.500" mr={2} />}
      <Heading size="md" fontWeight="bold">{title}</Heading>
      {tooltip && (
        <Tooltip label={tooltip} hasArrow placement="right">
          <Icon as={FaInfoCircle} ml={2} boxSize={4} color="gray.400" cursor="help" />
        </Tooltip>
      )}
    </Flex>
  );
};

export default SectionHeader;