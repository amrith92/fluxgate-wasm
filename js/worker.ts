import { createFluxgate } from './index.js';
import type { FluxgateInit, WorkerFluxgate } from './types.js';

export async function createWorkerGate(init: FluxgateInit): Promise<WorkerFluxgate> {
  const gate = await createFluxgate(init);
  return {
    ...gate,
    async terminate(): Promise<void> {
      // Placeholder worker API: since the current build is single-threaded we do
      // not spawn an actual worker. The method exists for future compatibility.
    },
  };
}
