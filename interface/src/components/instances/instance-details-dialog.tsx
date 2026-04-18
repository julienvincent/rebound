import { Clock3, Server } from 'lucide-react';

import { DetailList } from '@/components/instances/detail-list';
import { MetricRow } from '@/components/instances/metric-row';
import { StatusBadge } from '@/components/instances/status-badge';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import {
  format_currency,
  format_instance_duration,
  format_memory,
  format_nullable_timestamp,
  format_relative_time,
  format_seconds,
} from '@/lib/instance-formatters';
import type { SchemaInstance, SchemaRegion } from '@julienvincent/rebound-client/types';

export function InstanceDetailsDialog({
  instance,
  now,
  region,
  open,
  on_open_change,
}: {
  instance: SchemaInstance | null;
  now: Date;
  region?: SchemaRegion;
  open: boolean;
  on_open_change: (open: boolean) => void;
}) {
  return (
    <Dialog open={open} onOpenChange={on_open_change}>
      <DialogContent className="max-w-md gap-0 overflow-hidden p-0">
        {instance ? (
          <>
            <DialogHeader className="px-4 pt-4 pb-3 text-left">
              <DialogTitle>{instance.fly_app_name}</DialogTitle>
              <DialogDescription>{instance.id}</DialogDescription>
            </DialogHeader>

            <div className="space-y-4 px-4 pb-4">
              <div className="flex items-center justify-between gap-3 rounded-xl border border-border/70 bg-muted/30 px-3 py-3">
                <div>
                  <p className="text-sm font-medium">{region?.name ?? instance.region}</p>
                  <p className="text-xs text-muted-foreground">{instance.region}</p>
                </div>
                <StatusBadge status={instance.status} />
              </div>

              <div className="grid grid-cols-2 gap-2">
                <MetricRow
                  icon={Clock3}
                  label="Created"
                  value={format_relative_time(instance.created_at, now)}
                />
                <MetricRow label="Duration" value={format_instance_duration(instance, now)} />
                <MetricRow
                  icon={Server}
                  label="Memory"
                  value={format_memory(instance.vm_memory_mb)}
                />
                <MetricRow label="Max time" value={format_seconds(instance.max_duration_secs)} />
                <MetricRow label="Cost" value={format_currency(instance.cost_usd)} />
                <MetricRow
                  label="Rate"
                  value={`${format_currency(instance.cost_per_sec_usd)}/s`}
                />
              </div>

              <DetailList
                items={[
                  { label: 'Tailscale node', value: instance.tailscale_node_name },
                  { label: 'Fly machine id', value: instance.fly_machine_id },
                  { label: 'Tailscale device id', value: instance.tailscale_device_id },
                  {
                    label: 'Started at',
                    value: format_nullable_timestamp(instance.started_at),
                  },
                  {
                    label: 'Stopped at',
                    value: format_nullable_timestamp(instance.stopped_at),
                  },
                  { label: 'Stop reason', value: instance.stop_reason },
                  { label: 'Error', value: instance.error },
                ]}
              />
            </div>
          </>
        ) : null}
      </DialogContent>
    </Dialog>
  );
}
