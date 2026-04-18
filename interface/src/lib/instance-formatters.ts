import { formatDistanceStrict } from 'date-fns';

import { get_status_tone } from '@/lib/instance-status';
import type { SchemaInstance } from '@julienvincent/rebound-client/types';

export function format_relative_time(value: string, now: Date) {
  return formatDistanceStrict(new Date(value), now, {
    addSuffix: true,
    roundingMethod: 'floor',
  });
}

export function format_memory(value: number) {
  if (value >= 1024) {
    return `${value / 1024} GB`;
  }

  return `${value} MB`;
}

export function format_currency(value: number) {
  return new Intl.NumberFormat('en-US', {
    style: 'currency',
    currency: 'USD',
    minimumFractionDigits: value < 0.01 ? 4 : 2,
    maximumFractionDigits: value < 0.01 ? 4 : 2,
  }).format(value);
}

export function format_instance_duration(instance: SchemaInstance, now: Date) {
  const status_tone = get_status_tone(instance.status);

  if (status_tone === 'yellow' || !instance.started_at) {
    return '-';
  }

  if (status_tone === 'green') {
    return formatDistanceStrict(new Date(instance.started_at), now, {
      unit: 'second',
      roundingMethod: 'floor',
    });
  }

  return format_seconds(instance.duration_secs);
}

export function format_seconds(total_seconds: number) {
  const safe_seconds = Math.max(0, Math.floor(total_seconds));
  const hours = Math.floor(safe_seconds / 3600);
  const minutes = Math.floor((safe_seconds % 3600) / 60);
  const seconds = safe_seconds % 60;

  if (hours > 0) {
    return `${hours}h ${minutes}m`;
  }

  if (minutes > 0) {
    return `${minutes}m ${seconds}s`;
  }

  return `${seconds}s`;
}

export function format_nullable_timestamp(value: string | null | undefined) {
  if (!value) {
    return null;
  }

  const date = new Date(value);
  return `${date.toLocaleDateString()} ${date.toLocaleTimeString([], {
    hour: 'numeric',
    minute: '2-digit',
    second: '2-digit',
  })}`;
}
