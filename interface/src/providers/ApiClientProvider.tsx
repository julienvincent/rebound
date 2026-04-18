import React, { createContext, useMemo } from 'react';
import createClient from 'openapi-react-query';

import {
  createFetchClient,
  type FetchClient,
} from '@julienvincent/rebound-client';
import { use_config } from '@/hooks/use-config';

function create_client(fetchClient: FetchClient) {
  return createClient(fetchClient);
}

export type ReactQueryClient = ReturnType<typeof create_client>;

type ApiContextValue = {
  fetch_client: FetchClient;
  query_client: ReactQueryClient;
};
export const ApiContext = createContext<ApiContextValue | null>(null);

export function APIClientProvider({ children }: React.PropsWithChildren<{}>) {
  const config = use_config();

  const context_value = useMemo(() => {
    const headers: Record<string, string> = {};

    let fetch_client = createFetchClient({
      baseUrl: config.api_endpoint,
      headers,
    });
    let query_client = create_client(fetch_client);
    return {
      fetch_client,
      query_client,
    };
  }, []);

  return (
    <ApiContext.Provider value={context_value}>{children}</ApiContext.Provider>
  );
}
