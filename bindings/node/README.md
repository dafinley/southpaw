# Node/Bun Binding Skeleton

This package exposes a `checkPosture()` API intended for Node.js and Bun.

Current state:

- JS wrapper API shape is in place
- Native addon Rust crate exists in `bindings/node/native`
- Packaging/prebuild pipeline is TODO

Example:

```js
import { checkPosture } from "@southpaw/southpaw";

const result = await checkPosture({ policyFile: "./southpaw.yaml" });
if (result.status === "fail") process.exit(1);
```
