# Go Binding (CLI Wrapper)

This module provides a Go API that executes the `southpaw` CLI and parses its JSON report.

It avoids CGO and keeps the Rust core/CLI as the primary implementation.

## Example

```go
ctx := context.Background()
report, err := southpaw.Check(ctx, southpaw.Options{
    PolicyFile: "southpaw.yaml",
    Mode:       southpaw.ModeFail,
})
if err != nil {
    log.Fatal(err)
}
if report.Status == southpaw.StatusFail {
    log.Fatal("southpaw check failed")
}
```
