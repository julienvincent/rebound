import { Star } from 'lucide-react';

import { Button } from '@/components/ui/button';
import { CommandItem } from '@/components/ui/command';
import type { SchemaRegion } from '@julienvincent/rebound-client/types';

export function RegionItem({
  region,
  is_favourite,
  on_select,
  on_toggle_favourite,
}: {
  region: SchemaRegion;
  is_favourite: boolean;
  on_select: (value: string) => void;
  on_toggle_favourite: (value: string) => void;
}) {
  return (
    <CommandItem
      value={`${region.code} ${region.name}`}
      className="flex items-center gap-3 rounded-lg px-3 py-3"
      onSelect={() => on_select(region.code)}
    >
      <div className="min-w-0 flex-1">
        <p className="text-sm font-medium">{region.name}</p>
        <p className="text-xs text-muted-foreground">{region.code}</p>
      </div>

      <Button
        type="button"
        size="icon-sm"
        variant="ghost"
        className="size-8 rounded-full"
        onClick={(event) => {
          event.preventDefault();
          event.stopPropagation();
          on_toggle_favourite(region.code);
        }}
      >
        <Star className={is_favourite ? 'size-4 fill-current text-amber-500' : 'size-4'} />
      </Button>
    </CommandItem>
  );
}
