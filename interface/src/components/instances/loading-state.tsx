import { LoaderCircle } from 'lucide-react';

export function LoadingState({ label }: { label: string }) {
  return (
    <div className="flex min-h-32 flex-col items-center justify-center gap-2 rounded-xl border border-dashed border-border/70 bg-muted/30 text-sm text-muted-foreground">
      <LoaderCircle className="size-4 animate-spin" />
      <p>{label}</p>
    </div>
  );
}
