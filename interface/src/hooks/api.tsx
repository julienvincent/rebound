import { useContext } from 'react';
import {
  ApiContext,
  type ReactQueryClient,
} from '@/providers/ApiClientProvider';
import type { FetchClient } from '@julienvincent/rebound-client';

export function use_client(): FetchClient {
  const context = useContext(ApiContext);
  if (!context) {
    throw new Error('useAPI must be used within a APIProvider.');
  }
  return context.fetch_client;
}

export function use_query_api(): ReactQueryClient {
  const context = useContext(ApiContext);
  if (!context) {
    throw new Error('useAPI must be used within a APIProvider.');
  }
  return context.query_client;
}
