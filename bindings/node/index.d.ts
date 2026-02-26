export type PostureStatus = "pass" | "warn" | "fail" | "degraded";

export interface Finding {
  id: string;
  section: string;
  message: string;
  severity: "info" | "warn" | "fail";
  score: number;
  details?: Record<string, string>;
}

export interface PostureReport {
  status: PostureStatus;
  score: number;
  findings: Finding[];
  identity: Record<string, unknown>;
  filesystem: Record<string, unknown>;
  sandbox: Record<string, unknown>;
  network: Record<string, unknown>;
}

export interface CheckPostureOptions {
  policyFile?: string;
}

export declare function checkPosture(options?: CheckPostureOptions): Promise<PostureReport>;

