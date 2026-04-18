import './index.css';

import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';

import { RouterProvider, createRouter } from '@tanstack/react-router';

import { routeTree } from './tree.gen';
import { QueryProvider } from './providers/QueryProvider';
import { APIClientProvider } from './providers/ApiClientProvider';
import { ConfigProvider } from './providers/ConfigProvider';

const router = createRouter({
  routeTree,
  trailingSlash: 'preserve',
  scrollRestoration: true,
});
declare module '@tanstack/react-router' {
  interface Register {
    router: typeof router;
  }

  interface StaticDataRouteOption {
    meta?: {};
  }
}

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <ConfigProvider>
      <QueryProvider>
        <APIClientProvider>
          <RouterProvider router={router} />
        </APIClientProvider>
      </QueryProvider>
    </ConfigProvider>
  </StrictMode>,
);
