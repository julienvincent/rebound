import { Badge } from '@/components/ui/badge';
import { cn } from '@/lib/utils';
import { get_status_tone } from '@/lib/instance-status';

export function StatusBadge({ status }: { status: string }) {
  const tone = get_status_tone(status);

  return (
    <Badge
      className={cn(
        'border px-2.5 py-1 text-xs font-medium capitalize',
        tone === 'green' && 'border-emerald-500/30 bg-emerald-500/15 text-emerald-300',
        tone === 'yellow' && 'animate-pulse border-amber-500/30 bg-amber-500/15 text-amber-300',
        tone === 'red' && 'border-rose-500/30 bg-rose-500/15 text-rose-300',
        tone === 'neutral' && 'border-border bg-muted text-muted-foreground'
      )}
    >
      {status}
    </Badge>
  );
}
