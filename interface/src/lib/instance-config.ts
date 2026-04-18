export const favourite_regions_key = 'rebound:favourite_regions';

export const memory_options = [
  { value: '256', label: '256 MB', description: 'small' },
  { value: '512', label: '512 MB', description: 'default' },
  { value: '1024', label: '1 GB', description: 'balanced' },
  { value: '2048', label: '2 GB', description: 'more headroom' },
] as const;

export const duration_options = [
  { value: '1800', label: '30m', description: 'quick run' },
  { value: '3600', label: '1h', description: 'short session' },
  { value: '10800', label: '3h', description: 'extended' },
  { value: '21600', label: '6h', description: 'default' },
] as const;
