import { createRootRoute, Outlet } from '@tanstack/react-router';

export type NavLink = {
  label: string;
  to: string;
};

const RootLayout = () => {
  return <Outlet />;
};

export const Route = createRootRoute({ component: RootLayout });
