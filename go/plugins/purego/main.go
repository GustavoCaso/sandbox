package main

import (
	"fmt"
	"os"

	"github.com/ebitengine/purego"
)

// Define the Go struct
type Result struct {
	Message string
	Len     int
}

func main() {
	library := os.Args[1]

	// Open shared library
	libc, err := purego.Dlopen(library, purego.RTLD_NOW|purego.RTLD_GLOBAL)
	if err != nil {
		panic(err)
	}

	var run func() *Result
	purego.RegisterLibFunc(&run, libc, "Run")

	var free func(*Result)
	purego.RegisterLibFunc(&free, libc, "FreeResult")

	result := run()

	if result != nil {
		fmt.Println("Len:", result.Len) // No sure why the Len is 0
		fmt.Println("Result:", result.Message)
		free(result)
	}
}
