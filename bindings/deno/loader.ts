export type SouthpawWasmModule = {
  check_posture_json_from_policy_text: (
    policyText: string,
    formatHint?: string | null,
  ) => string;
};

export async function loadWasmBindings(): Promise<SouthpawWasmModule> {
  try {
    const mod = await import("./pkg/southpaw_deno_wasm.js");
    if (typeof mod.default === "function") {
      await mod.default();
    }
    if (typeof mod.check_posture_json_from_policy_text !== "function") {
      throw new Error("Generated WASM module is missing check_posture_json_from_policy_text");
    }
    return mod as SouthpawWasmModule;
  } catch (err) {
    throw new Error(
      "Deno WASM bindings are not built yet. Build the Rust WASM crate and generate ./pkg assets. " +
        String(err),
    );
  }
}
