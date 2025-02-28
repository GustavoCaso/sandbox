package main

/*
#include <stdio.h>
#include <stdlib.h>

typedef struct
{
  char *Char;
  int Len;
} Result;
*/
import "C"

import (
	"unsafe"
)

// Define the Go struct
type Result struct {
	Message string
	Len     int
}

//export Run
func Run() *C.Result {
	// Create the Go struct with data
	res := Result{
		Message: "Hello from Go shared library!",
		Len:     30,
	}

	// Allocate memory for the C struct
	cRes := (*C.Result)(C.malloc(C.size_t(unsafe.Sizeof(C.Result{}))))
	if cRes == nil {
		return nil
	}

	// Convert Go string to C string
	cRes.Char = C.CString(res.Message)
	cRes.Len = C.int(res.Len)

	return cRes
}

//export FreeResult
func FreeResult(res *C.Result) {
	if res != nil {
		C.free(unsafe.Pointer(res.Char)) // Free C string
		C.free(unsafe.Pointer(res))      // Free struct memory
	}
}

func main() {}

// go build -o libresult.so -buildmode=c-shared *.go
