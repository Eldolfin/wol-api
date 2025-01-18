export const asyncSleep = (time_ms: number) =>
  new Promise((resolve) => setTimeout(resolve, time_ms));
