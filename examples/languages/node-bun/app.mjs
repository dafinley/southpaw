console.log(JSON.stringify({
  demo: "node-bun",
  runtime: typeof Bun === "undefined" ? "node" : "bun",
  status: "started",
}));

