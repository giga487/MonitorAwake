// All callers share one drain: a debounce, Close and Quit cannot race writes.
export function createSaveQueue(
  save: () => Promise<boolean>,
  dirty: () => boolean,
) {
  let pending: Promise<boolean> | null = null;
  return () => {
    if (pending) return pending;
    const task = (async () => {
      while (dirty()) {
        if (!(await save())) return false;
      }
      return true;
    })();
    pending = task.finally(() => {
      pending = null;
    });
    return pending;
  };
}
