import { useEffect, useState } from 'react';

export function useSystemTheme(): 'light' | 'dark' {
  const [mode, setMode] = useState<'light' | 'dark'>('dark');

  useEffect(() => {
    if (typeof window !== 'undefined') {
      const mq = window.matchMedia('(prefers-color-scheme: dark)');
      setMode(mq.matches ? 'dark' : 'light');
      const onMq = () => setMode(mq.matches ? 'dark' : 'light');
      mq.addEventListener?.('change', onMq);
      // @ts-ignore
      mq.addListener?.(onMq);
      return () => {
        mq.removeEventListener?.('change', onMq);
        // @ts-ignore
        mq.removeListener?.(onMq);
      };
    }
  }, []);

  return mode;
}