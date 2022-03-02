import { useEffect } from 'react';

export const useUnmount = (fn: () => void) => {
  if (process.env.NODE_ENV === 'development') {
    if (typeof fn !== 'function') {
      console.error(`useMount: parameter \`fn\` expected to be a function, but got "${typeof fn}".`);
    }
  }

  useEffect(() => {
    return fn;
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);
};
