import { Clock3, LoaderCircle, MapPin, Server, Square } from 'lucide-react';

import { Button } from '@/components/ui/button';
import { MetricRow } from '@/components/instances/metric-row';
import { StatusBadge } from '@/components/instances/status-badge';
import { format_instance_duration, format_memory, format_relative_time } from '@/lib/instance-formatters';
import { get_status_tone } from '@/lib/instance-status';
import type { SchemaInstance } from '@julienvincent/rebound-client/types';

export function InstanceCard({
  instance,
  now,
  region_name,
  is_stopping,
  on_open_details,
  on_stop,
}: {
  instance: SchemaInstance;
  now: Date;
  region_name?: string;
  is_stopping: boolean;
  on_open_details: (id: string) => void;
  on_stop: (id: string) => void;
}) {
  const status_tone = get_status_tone(instance.status);
  const can_stop = status_tone !== 'red';

  return (
    <div
      role="button"
      tabIndex={0}
      className="w-full rounded-xl border border-border/70 bg-background/80 p-3 text-left shadow-xs transition-colors hover:bg-muted/20"
      onClick={() => on_open_details(instance.id)}
      onKeyDown={(event) => {
        if (event.key === 'Enter' || event.key === ' ') {
          event.preventDefault();
          on_open_details(instance.id);
        }
      }}
    >
      <div className="flex items-start justify-between gap-3">
        <div className="min-w-0 space-y-1">
          <p className="truncate text-sm font-medium">{instance.fly_app_name}</p>
          <p className="truncate text-xs text-muted-foreground">{instance.id}</p>
        </div>
        <StatusBadge status={instance.status} />
      </div>

      <div className="mt-3 grid gap-2 text-sm sm:grid-cols-2">
        <MetricRow icon={MapPin} label="Region" value={region_name ?? instance.region} />
        <MetricRow icon={Server} label="Memory" value={format_memory(instance.vm_memory_mb)} />
        <MetricRow
          icon={Clock3}
          label="Created"
          value={format_relative_time(instance.created_at, now)}
        />
        <MetricRow label="Duration" value={format_instance_duration(instance, now)} />
      </div>

      <div className="mt-3 rounded-lg bg-muted/50 px-3 py-2 text-xs text-muted-foreground">
        Node {instance.tailscale_node_name}
      </div>

      {can_stop ? (
        <Button
          variant="outline"
          className="mt-3 w-full justify-center"
          onClick={(event) => {
            event.stopPropagation();
            on_stop(instance.id);
          }}
          disabled={is_stopping}
        >
          {is_stopping ? (
            <>
              <LoaderCircle className="size-4 animate-spin" />
              Stopping instance
            </>
          ) : (
            <>
              <Square className="size-4" />
              Stop instance
            </>
          )}
        </Button>
      ) : null}
    </div>
  );
}
