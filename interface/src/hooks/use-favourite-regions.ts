import { useEffect, useState } from 'react';

import { favourite_regions_key } from '@/lib/instance-config';

export function use_favourite_regions() {
  const [favourite_regions, set_favourite_regions] = useState<string[]>([]);

  useEffect(() => {
    const stored_value = window.localStorage.getItem(favourite_regions_key);
    if (!stored_value) {
      return;
    }

    try {
      const parsed = JSON.parse(stored_value);
      if (Array.isArray(parsed)) {
        set_favourite_regions(
          parsed.filter((value): value is string => typeof value === 'string')
        );
      }
    } catch {
      window.localStorage.removeItem(favourite_regions_key);
    }
  }, []);

  function toggle_favourite_region(code: string) {
    set_favourite_regions((current_value) => {
      const next_value = current_value.includes(code)
        ? current_value.filter((current_code) => current_code !== code)
        : [...current_value, code];

      window.localStorage.setItem(
        favourite_regions_key,
        JSON.stringify(next_value)
      );

      return next_value;
    });
  }

  return {
    favourite_regions,
    toggle_favourite_region,
  };
}
