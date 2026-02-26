package main

import (
	"context"
	"fmt"
	"log"

	"github.com/your-org/southpaw/bindings/go/southpaw"
)

func main() {
	report, err := southpaw.Require(context.Background(), southpaw.Options{
		PolicyFile: "southpaw.yaml",
	})
	if err != nil {
		log.Fatal(err)
	}

	fmt.Printf("demo=go status=%s\n", report.Status)
}
