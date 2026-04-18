export type InstanceStatusTone = 'green' | 'yellow' | 'red' | 'neutral';

export function get_status_tone(status: string): InstanceStatusTone {
  const normalized_status = status.toLowerCase();

  if (normalized_status === 'running') {
    return 'green';
  }

  if (
    normalized_status.includes('launch') ||
    normalized_status.includes('start') ||
    normalized_status.includes('creat') ||
    normalized_status.includes('pend') ||
    normalized_status.includes('provision')
  ) {
    return 'yellow';
  }

  if (
    normalized_status.includes('stop') ||
    normalized_status.includes('fail') ||
    normalized_status.includes('error') ||
    normalized_status.includes('destroy')
  ) {
    return 'red';
  }

  return 'neutral';
}

export function is_transient_status(status: string) {
  return get_status_tone(status) === 'yellow';
}
