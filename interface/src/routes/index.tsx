import { useEffect, useMemo, useState } from 'react';
import { useQueryClient } from '@tanstack/react-query';
import { createFileRoute } from '@tanstack/react-router';

import { ConfigurationDialog } from '@/components/create-instance/configuration-dialog';
import { RegionPickerDialog } from '@/components/create-instance/region-picker-dialog';
import { EmptyState } from '@/components/instances/empty-state';
import { InstanceCard } from '@/components/instances/instance-card';
import { InstanceDetailsDialog } from '@/components/instances/instance-details-dialog';
import { LoadingState } from '@/components/instances/loading-state';
import { Button } from '@/components/ui/button';
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@/components/ui/card';
import { use_query_api } from '@/hooks/api';
import { use_favourite_regions } from '@/hooks/use-favourite-regions';
import { use_now } from '@/hooks/use-now';
import { is_transient_status } from '@/lib/instance-status';
import type {
  SchemaCreateResponse,
  SchemaInstance,
  SchemaListResponse,
  SchemaRegion,
} from '@julienvincent/rebound-client/types';

export const Route = createFileRoute('/')({
  component: RouteComponent,
});

function RouteComponent() {
  const query_api = use_query_api();
  const query_client = useQueryClient();
  const now = use_now();
  const { favourite_regions, toggle_favourite_region } = use_favourite_regions();

  const [region_dialog_open, set_region_dialog_open] = useState(false);
  const [config_dialog_open, set_config_dialog_open] = useState(false);
  const [selected_region, set_selected_region] = useState<string | null>(null);
  const [selected_memory_mb, set_selected_memory_mb] = useState('512');
  const [selected_duration_secs, set_selected_duration_secs] = useState('21600');
  const [selected_instance_id, set_selected_instance_id] = useState<string | null>(null);
  const [page_cursors, set_page_cursors] = useState<Array<string | null>>([null]);
  const [page_index, set_page_index] = useState(0);

  const current_cursor = page_cursors[page_index] ?? null;

  const list_request = useMemo(
    () => ({
      body: {
        limit: 5,
        cursor: current_cursor,
      },
    }),
    [current_cursor]
  );

  const instances_query = query_api.useQuery('post', '/instance/list', list_request);
  const regions_query = query_api.useQuery('post', '/region/list');

  const get_first_page_query_key = () =>
    query_api.queryOptions('post', '/instance/list', {
      body: {
        limit: 5,
      },
    }).queryKey;

  const refresh_instances = async () => {
    await query_client.invalidateQueries({
      queryKey: ['post', '/instance/list'],
    });
  };

  const create_instance = query_api.useMutation('post', '/instance/create', {
    onSuccess: async (response) => {
      set_page_cursors([null]);
      set_page_index(0);

      query_client.setQueryData<SchemaListResponse>(
        get_first_page_query_key(),
        (current_value) => {
          const optimistic_instance = create_optimistic_instance(
            response,
            Number(selected_duration_secs)
          );

          if (!current_value) {
            return {
              instances: [optimistic_instance],
              next_cursor: null,
            };
          }

          const remaining_instances = current_value.instances.filter(
            (instance) => instance.id !== optimistic_instance.id
          );

          return {
            ...current_value,
            instances: [optimistic_instance, ...remaining_instances],
          };
        }
      );

      await refresh_instances();
      set_config_dialog_open(false);
    },
  });

  const stop_instance = query_api.useMutation('post', '/instance/stop', {
    onSuccess: refresh_instances,
  });

  const region_lookup = useMemo(() => {
    return new Map((regions_query.data?.regions ?? []).map((region) => [region.code, region]));
  }, [regions_query.data?.regions]);

  const favourite_region_items = useMemo(() => {
    return favourite_regions
      .map((code) => region_lookup.get(code))
      .filter((region): region is SchemaRegion => Boolean(region));
  }, [favourite_regions, region_lookup]);

  const other_region_items = useMemo(() => {
    const favourite_set = new Set(favourite_regions);
    return (regions_query.data?.regions ?? []).filter(
      (region) => !favourite_set.has(region.code)
    );
  }, [favourite_regions, regions_query.data?.regions]);

  const selected_region_item = selected_region ? region_lookup.get(selected_region) ?? null : null;
  const selected_instance =
    instances_query.data?.instances.find((instance) => instance.id === selected_instance_id) ??
    null;
  const has_next_page = Boolean(instances_query.data?.next_cursor);
  const has_previous_page = page_index > 0;

  const should_poll_instances = (instances_query.data?.instances ?? []).some((instance) =>
    is_transient_status(instance.status)
  );

  useEffect(() => {
    if (!should_poll_instances) {
      return;
    }

    const interval_id = window.setInterval(() => {
      void instances_query.refetch();
    }, 1000);

    return () => {
      window.clearInterval(interval_id);
    };
  }, [instances_query, should_poll_instances]);

  function open_region_dialog() {
    set_region_dialog_open(true);
  }

  function choose_region(code: string) {
    set_selected_region(code);
    set_region_dialog_open(false);
    set_config_dialog_open(true);
  }

  function back_to_region_dialog() {
    set_config_dialog_open(false);
    set_region_dialog_open(true);
  }

  function close_config_dialog(open: boolean) {
    set_config_dialog_open(open);
    if (!open) {
      set_selected_memory_mb('512');
      set_selected_duration_secs('21600');
    }
  }

  function submit_create() {
    if (!selected_region || create_instance.isPending) {
      return;
    }

    create_instance.mutate({
      body: {
        region: selected_region,
        max_duration_secs: Number(selected_duration_secs),
        vm_memory_mb: Number(selected_memory_mb),
      },
    });
  }

  function stop_instance_by_id(id: string) {
    if (stop_instance.isPending) {
      return;
    }

    stop_instance.mutate({
      body: {
        id,
      },
    });
  }

  function go_to_next_page() {
    const next_cursor = instances_query.data?.next_cursor;
    if (!next_cursor) {
      return;
    }

    set_page_cursors((current_value) => {
      const next_value = current_value.slice(0, page_index + 1);
      next_value[page_index + 1] = next_cursor;
      return next_value;
    });
    set_page_index((current_value) => current_value + 1);
  }

  function go_to_previous_page() {
    if (!has_previous_page) {
      return;
    }

    set_page_index((current_value) => Math.max(0, current_value - 1));
  }

  return (
    <main className="min-h-screen bg-[radial-gradient(circle_at_top,_rgba(110,231,183,0.08),_transparent_25%),linear-gradient(180deg,_var(--background)_0%,_color-mix(in_oklab,_var(--background)_90%,_var(--muted))_100%)] px-4 py-4 sm:px-6">
      <div className="mx-auto flex w-full max-w-md flex-col gap-4 pb-8">
        <Button className="h-11 w-full" onClick={open_region_dialog}>
          Create instance
        </Button>

        <Card className="border-border/70 bg-card/95 shadow-sm">
          <CardHeader className="pb-3">
            <CardTitle className="text-base">Instances</CardTitle>
            <CardDescription>
              {instances_query.isLoading
                ? 'Loading current machines'
                : `${instances_query.data?.instances.length ?? 0} recent instance${
                    instances_query.data?.instances.length === 1 ? '' : 's'
                  }`}
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-3">
            {instances_query.isLoading ? (
              <LoadingState label="Loading instances" />
            ) : instances_query.error ? (
              <EmptyState
                title="Could not load instances"
                description="Check the API connection and try again."
              />
            ) : instances_query.data?.instances.length ? (
              <>
                {instances_query.data.instances.map((instance) => (
                  <InstanceCard
                    key={instance.id}
                    instance={instance}
                    now={now}
                    region_name={region_lookup.get(instance.region)?.name}
                    is_stopping={
                      stop_instance.isPending && stop_instance.variables?.body.id === instance.id
                    }
                    on_open_details={set_selected_instance_id}
                    on_stop={stop_instance_by_id}
                  />
                ))}

                <div className="flex items-center justify-between gap-3 border-t border-border/70 pt-3">
                  <Button
                    variant="outline"
                    onClick={go_to_previous_page}
                    disabled={!has_previous_page || instances_query.isFetching}
                  >
                    Previous
                  </Button>
                  <p className="text-sm text-muted-foreground">Page {page_index + 1}</p>
                  <Button
                    variant="outline"
                    onClick={go_to_next_page}
                    disabled={!has_next_page || instances_query.isFetching}
                  >
                    Next
                  </Button>
                </div>
              </>
            ) : (
              <EmptyState
                title="No instances yet"
                description="Create a new instance to start a session."
              />
            )}
          </CardContent>
        </Card>
      </div>

      <RegionPickerDialog
        open={region_dialog_open}
        favourite_regions={favourite_regions}
        favourite_region_items={favourite_region_items}
        other_region_items={other_region_items}
        on_open_change={set_region_dialog_open}
        on_select_region={choose_region}
        on_toggle_favourite={toggle_favourite_region}
      />

      <ConfigurationDialog
        open={config_dialog_open}
        region={selected_region_item}
        selected_region={selected_region}
        selected_memory_mb={selected_memory_mb}
        selected_duration_secs={selected_duration_secs}
        is_creating={create_instance.isPending}
        has_error={Boolean(create_instance.error)}
        on_open_change={close_config_dialog}
        on_back={back_to_region_dialog}
        on_memory_change={set_selected_memory_mb}
        on_duration_change={set_selected_duration_secs}
        on_submit={submit_create}
      />

      <InstanceDetailsDialog
        instance={selected_instance}
        now={now}
        region={selected_instance ? region_lookup.get(selected_instance.region) : undefined}
        open={Boolean(selected_instance)}
        on_open_change={(open) => {
          if (!open) {
            set_selected_instance_id(null);
          }
        }}
      />
    </main>
  );
}

function create_optimistic_instance(
  response: SchemaCreateResponse,
  max_duration_secs: number
): SchemaInstance {
  return {
    id: response.id,
    status: response.status,
    region: response.region,
    vm_memory_mb: response.vm_memory_mb,
    fly_app_name: response.fly_app_name,
    fly_machine_id: null,
    tailscale_device_id: null,
    tailscale_node_name: response.tailscale_node_name,
    cost_per_sec_usd: 0,
    max_duration_secs,
    created_at: response.created_at,
    started_at: null,
    stopped_at: null,
    stop_reason: null,
    error: null,
    cost_usd: 0,
    duration_secs: 0,
  };
}
