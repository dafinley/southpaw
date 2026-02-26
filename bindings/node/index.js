import { createRequire } from "node:module";

const require = createRequire(import.meta.url);

function loadNativeBinding() {
  const candidates = [
    "./native/index.node",
    "./index.node"
  ];

  const errors = [];
  for (const candidate of candidates) {
    try {
      return require(candidate);
    } catch (err) {
      errors.push(String(err));
    }
  }

  throw new Error(
    "southpaw native addon is not built yet.\n\n" +
      "To build it:\n" +
      "  cd bindings/node/native && cargo build --release\n" +
      "  # or: npm run build (if configured)\n\n" +
      "Expected one of: " + candidates.join(", ") + "\n" +
      "See bindings/node/README.md for details.\n\n" +
      "Errors: " + errors.join(" | ")
  );
}

export async function checkPosture(options = {}) {
  const native = loadNativeBinding();
  const policyFile = options.policyFile ?? null;
  const json = native.checkPostureJson(policyFile);
  return JSON.parse(json);
}

