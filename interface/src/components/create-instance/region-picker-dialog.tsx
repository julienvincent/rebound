import { Star } from 'lucide-react';

import { RegionItem } from '@/components/create-instance/region-item';
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandList,
} from '@/components/ui/command';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { Button } from '@/components/ui/button';
import type { SchemaRegion } from '@julienvincent/rebound-client/types';

export function RegionPickerDialog({
  open,
  favourite_regions,
  favourite_region_items,
  other_region_items,
  on_open_change,
  on_select_region,
  on_toggle_favourite,
}: {
  open: boolean;
  favourite_regions: string[];
  favourite_region_items: SchemaRegion[];
  other_region_items: SchemaRegion[];
  on_open_change: (open: boolean) => void;
  on_select_region: (code: string) => void;
  on_toggle_favourite: (code: string) => void;
}) {
  return (
    <Dialog open={open} onOpenChange={on_open_change}>
      <DialogContent className="max-w-md gap-0 overflow-hidden p-0">
        <DialogHeader className="px-4 pt-4 pb-3 text-left">
          <DialogTitle>Select a region</DialogTitle>
          <DialogDescription>
            Pick where the new instance should launch.
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-4 px-4 pb-4">
          {favourite_region_items.length ? (
            <div className="space-y-2">
              <p className="text-sm font-medium">Favourites</p>
              <div className="flex flex-wrap gap-2">
                {favourite_region_items.map((region) => (
                  <Button
                    key={region.code}
                    type="button"
                    variant="outline"
                    className="h-9 rounded-full px-3"
                    onClick={() => on_select_region(region.code)}
                  >
                    <Star className="size-4 fill-current text-amber-500" />
                    {region.name}
                  </Button>
                ))}
              </div>
            </div>
          ) : null}

          <div className="overflow-hidden rounded-xl border border-border/70">
            <Command className="bg-transparent">
              <CommandInput placeholder="Search regions" className="h-11" />
              <CommandList className="max-h-[50vh]">
                <CommandEmpty>No matching regions.</CommandEmpty>

                {favourite_region_items.length ? (
                  <CommandGroup heading="Favourites">
                    {favourite_region_items.map((region) => (
                      <RegionItem
                        key={region.code}
                        region={region}
                        is_favourite
                        on_select={on_select_region}
                        on_toggle_favourite={on_toggle_favourite}
                      />
                    ))}
                  </CommandGroup>
                ) : null}

                <CommandGroup heading="All regions">
                  {other_region_items.map((region) => (
                    <RegionItem
                      key={region.code}
                      region={region}
                      is_favourite={favourite_regions.includes(region.code)}
                      on_select={on_select_region}
                      on_toggle_favourite={on_toggle_favourite}
                    />
                  ))}
                </CommandGroup>
              </CommandList>
            </Command>
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
}
