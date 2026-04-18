import { ConfigContext, type Config } from '@/providers/ConfigProvider';
import React from 'react';

export function use_config(): Config {
  const ctx = React.useContext(ConfigContext);
  if (!ctx) {
    throw new Error('No config loaded');
  }
  return ctx;
}
