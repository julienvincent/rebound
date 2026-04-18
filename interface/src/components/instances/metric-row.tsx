import { MapPin } from 'lucide-react';

export function MetricRow({
  icon: Icon,
  label,
  value,
}: {
  icon?: typeof MapPin;
  label: string;
  value: string;
}) {
  return (
    <div className="rounded-lg bg-muted/40 px-3 py-2">
      <div className="mb-1 flex items-center gap-2 text-xs uppercase tracking-[0.16em] text-muted-foreground">
        {Icon ? <Icon className="size-3.5" /> : null}
        <span>{label}</span>
      </div>
      <p className="truncate text-sm font-medium">{value}</p>
    </div>
  );
}
