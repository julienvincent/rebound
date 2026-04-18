import { useEffect, useState } from 'react';

export function use_now() {
  const [now, set_now] = useState(() => new Date());

  useEffect(() => {
    const interval_id = window.setInterval(() => {
      set_now(new Date());
    }, 1000);

    return () => {
      window.clearInterval(interval_id);
    };
  }, []);

  return now;
}
