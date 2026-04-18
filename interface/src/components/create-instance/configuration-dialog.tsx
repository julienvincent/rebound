import { ArrowLeft, LoaderCircle } from 'lucide-react';

import { ConfigurationSection } from '@/components/create-instance/configuration-section';
import { Button } from '@/components/ui/button';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { ToggleGroup, ToggleGroupItem } from '@/components/ui/toggle-group';
import { duration_options, memory_options } from '@/lib/instance-config';
import type { SchemaRegion } from '@julienvincent/rebound-client/types';

export function ConfigurationDialog({
  open,
  region,
  selected_region,
  selected_memory_mb,
  selected_duration_secs,
  is_creating,
  has_error,
  on_open_change,
  on_back,
  on_memory_change,
  on_duration_change,
  on_submit,
}: {
  open: boolean;
  region: SchemaRegion | null;
  selected_region: string | null;
  selected_memory_mb: string;
  selected_duration_secs: string;
  is_creating: boolean;
  has_error: boolean;
  on_open_change: (open: boolean) => void;
  on_back: () => void;
  on_memory_change: (value: string) => void;
  on_duration_change: (value: string) => void;
  on_submit: () => void;
}) {
  return (
    <Dialog open={open} onOpenChange={on_open_change}>
      <DialogContent className="max-w-md gap-0 overflow-hidden p-0">
        <DialogHeader className="px-4 pt-4 pb-3 text-left">
          <div className="flex items-center gap-2">
            <Button size="icon-sm" variant="ghost" onClick={on_back}>
              <ArrowLeft className="size-4" />
            </Button>
            <div>
              <DialogTitle>Create instance</DialogTitle>
              <DialogDescription>
                {region
                  ? `${region.name} (${region.code})`
                  : 'Choose the configuration for this machine.'}
              </DialogDescription>
            </div>
          </div>
        </DialogHeader>

        <div className="space-y-5 px-4 pb-4">
          {has_error ? (
            <div className="rounded-lg border border-destructive/30 bg-destructive/5 px-3 py-2 text-sm text-destructive">
              Failed to create the instance. Please try again.
            </div>
          ) : null}

          <ConfigurationSection
            title="Memory"
            description="Quick machine sizes for mobile-first setup"
          >
            <ToggleGroup
              type="single"
              value={selected_memory_mb}
              onValueChange={(value) => value && on_memory_change(value)}
              variant="outline"
              className="grid w-full grid-cols-2 gap-2"
            >
              {memory_options.map((option) => (
                <ToggleGroupItem
                  key={option.value}
                  value={option.value}
                  className="h-auto min-h-16 flex-col items-start justify-center gap-1 rounded-xl border border-border/70 px-3 py-3 text-left"
                >
                  <span className="text-sm font-medium">{option.label}</span>
                  <span className="text-xs text-muted-foreground">{option.description}</span>
                </ToggleGroupItem>
              ))}
            </ToggleGroup>
          </ConfigurationSection>

          <ConfigurationSection
            title="Online time"
            description="How long the instance should stay available"
          >
            <ToggleGroup
              type="single"
              value={selected_duration_secs}
              onValueChange={(value) => value && on_duration_change(value)}
              variant="outline"
              className="grid w-full grid-cols-2 gap-2"
            >
              {duration_options.map((option) => (
                <ToggleGroupItem
                  key={option.value}
                  value={option.value}
                  className="h-auto min-h-16 flex-col items-start justify-center gap-1 rounded-xl border border-border/70 px-3 py-3 text-left"
                >
                  <span className="text-sm font-medium">{option.label}</span>
                  <span className="text-xs text-muted-foreground">{option.description}</span>
                </ToggleGroupItem>
              ))}
            </ToggleGroup>
          </ConfigurationSection>

          <div className="rounded-xl border border-border/70 bg-muted/40 px-3 py-3 text-sm text-muted-foreground">
            Launching in{' '}
            <span className="font-medium text-foreground">{region?.name ?? selected_region}</span>{' '}
            with{' '}
            <span className="font-medium text-foreground">
              {memory_options.find((option) => option.value === selected_memory_mb)?.label}
            </span>{' '}
            for{' '}
            <span className="font-medium text-foreground">
              {duration_options.find((option) => option.value === selected_duration_secs)?.label}
            </span>
            .
          </div>
        </div>

        <DialogFooter className="border-t border-border/70 px-4 py-4">
          <Button className="w-full" onClick={on_submit} disabled={!selected_region || is_creating}>
            {is_creating ? (
              <>
                <LoaderCircle className="size-4 animate-spin" />
                Creating instance
              </>
            ) : (
              'Launch instance'
            )}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
