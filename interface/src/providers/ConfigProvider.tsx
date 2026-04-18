import React from 'react';

export type Config = {
  api_endpoint: string;
};
export const ConfigContext = React.createContext<Config | null>(null);

export function ConfigProvider({ children }: React.PropsWithChildren<{}>) {
  const [config, setConfig] = React.useState<Config | null>(null);
  const [error, setError] = React.useState<string | null>(null);

  React.useEffect(() => {
    fetch('/config.json')
      .then((res) => {
        return res.json();
      })
      .then((raw: { api_endpoint: string; feature_flags?: string[] }) => {
        setConfig({
          api_endpoint: raw.api_endpoint,
        });
      })
      .catch((ex) => {
        console.log('Failed to load config', ex);
        setError('Bad');
      });
  }, []);

  const context_value = React.useMemo(() => {
    return config;
  }, [config]);

  if (error) {
    if (import.meta.env.DEV) {
      return (
        <p>
          Failed to load config. You need to create a <b>config.json</b> file in{' '}
          <code>/public</code>
        </p>
      );
    }

    return 'Load error. Please contact support if this persists';
  }

  if (!context_value) {
    return null;
  }

  return (
    <ConfigContext.Provider value={context_value}>
      {children}
    </ConfigContext.Provider>
  );
}
