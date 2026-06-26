export interface TraceOptions {
  color_precision: number;
  filter_speckle: number;
  corner_threshold: number;
}

export const defaultTraceOptions: TraceOptions = {
  color_precision: 6,
  filter_speckle: 4,
  corner_threshold: 60,
};

export type BackendId = "vtracer" | "live" | "starvector-1b" | "starvector-8b";

export type BackendState =
  | { type: "ready" }
  | { type: "not_installed" }
  | { type: "incompatible"; value: string }
  | { type: "installing"; value: number }
  | { type: "error"; value: string };

export interface BackendStatus {
  id: BackendId;
  state: BackendState;
}

export interface GpuInfo {
  name: string;
  vram_mb: number | null;
  compute_backend: "metal" | "cuda" | "rocm" | null;
}

export type InstallStage =
  | { kind: "python_env" }
  | { kind: "model_weights"; value: BackendId };

export interface InstallProgress {
  stage: InstallStage;
  bytes_downloaded: number;
  total_bytes: number;
}
