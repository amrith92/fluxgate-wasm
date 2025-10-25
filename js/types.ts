export type FluxgateInit = {
  policies?: FluxgatePolicy[];
  configText?: string;
  keySecret?: string;
  slices?: number;
  sketchWidth?: number;
  sketchDepth?: number;
  topK?: number;
  shardAHotCapacity?: number;
  admissionHitsToPromote?: number;
};

export type FluxgatePolicy = {
  id: string;
  match: string;
  limitPerSecond: number;
  burst: number;
  windowSeconds: number;
  action?: 'reject' | 'annotate';
};

export type CheckRequest = {
  ip?: string;
  route?: string;
  headers?: Record<string, string | undefined>;
  attrs?: Record<string, string | number | boolean | null | undefined>;
};

export type CheckResult = {
  allowed: boolean;
  retryAfterMs?: number;
  decisions: Record<string, { allowed: boolean; retryAfterMs?: number }>;
};

export interface Fluxgate {
  check(req: CheckRequest): CheckResult;
  checkBatch(reqs: CheckRequest[]): CheckResult[];
  rotate(): void;
  reload(cfg: FluxgateInit): void;
  snapshot(): Uint8Array;
  restore(bytes: Uint8Array): void;
  metrics(): Record<string, number>;
  version(): string;
}

export interface WorkerFluxgate extends Fluxgate {
  terminate(): Promise<void>;
}
