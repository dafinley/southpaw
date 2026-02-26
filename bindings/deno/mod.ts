import { loadWasmBindings } from "./loader.ts";

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
  policyText?: string;
  formatHint?: "yaml" | "yml" | "json";
}

export async function checkPosture(
  options: CheckPostureOptions = {},
): Promise<PostureReport> {
  const policyText = options.policyText ??
    (options.policyFile ? await Deno.readTextFile(options.policyFile) : undefined);

  if (!policyText) {
    throw new Error("Provide policyText or policyFile");
  }

  const wasm = await loadWasmBindings();
  const json = wasm.check_posture_json_from_policy_text(
    policyText,
    options.formatHint ?? inferFormatHint(options.policyFile ?? undefined),
  );

  return JSON.parse(json) as PostureReport;
}

function inferFormatHint(policyFile?: string): "yaml" | "json" | undefined {
  if (!policyFile) return undefined;
  if (policyFile.endsWith(".json")) return "json";
  if (policyFile.endsWith(".yaml") || policyFile.endsWith(".yml")) return "yaml";
  return undefined;
}

if (import.meta.main) {
  const report = await checkPosture({ policyFile: "./southpaw.yaml" });
  console.log(JSON.stringify(report, null, 2));
}

