export function DetailList({
  items,
}: {
  items: Array<{ label: string; value: string | null | undefined }>;
}) {
  const visible_items = items.filter((item) => item.value);

  return (
    <div className="rounded-xl border border-border/70 bg-muted/20">
      {visible_items.map((item) => (
        <div
          key={item.label}
          className="flex items-start justify-between gap-3 border-b border-border/70 px-3 py-3 last:border-b-0"
        >
          <p className="text-xs uppercase tracking-[0.16em] text-muted-foreground">
            {item.label}
          </p>
          <p className="max-w-[65%] break-words text-right text-sm font-medium">
            {item.value}
          </p>
        </div>
      ))}
    </div>
  );
}
