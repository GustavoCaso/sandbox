package main

/*
#include <stdio.h>
#include <stdlib.h>
#include <dlfcn.h>
#include "executor.h"

extern void* open_library(char *library)
{
	return dlopen(library, RTLD_LAZY);
}

extern void close_library(void *handle)
{
	dlclose(handle);
}


extern Result* run_check(void *handle)
{
	// Load the shared library
	if (!handle) {
			fprintf(stderr, "Error loading library:");
			return NULL;
	}

	// Load the function symbol
	Result* (*run)();
  *(void **)(&run) = dlsym(handle, "Run");

	// Check for errors
	char *error = dlerror();
	if (error) {
			fprintf(stderr, "Error loading symbol: %s\n", error);
			return NULL;
	}


	// Call the function and get the result
  Result *result = run();

	return result;
}

extern void free_result(void *handle, Result *result)
{

	// Load the shared library
	if (!handle) {
			fprintf(stderr, "Error loading library:");
	}

	// Load the function symbol
	void (*freeResult)(Result*);
  *(void **)(&freeResult) = dlsym(handle, "FreeResult");

	// Check for errors
	char *error = dlerror();
	if (error) {
			fprintf(stderr, "Error loading symbol: %s\n", error);
	}


	// Call the function
  freeResult(result);
}
*/
import "C"
import (
	"fmt"
	"os"
)

func main() {
	library := os.Args[1]

	// Open shared library
	handle := C.open_library(C.CString(library))

	result := C.run_check(handle)

	if result != nil {
		goStr := C.GoStringN(result.Char, C.int(result.Len))
		// Print the result output
		fmt.Println("Result:", goStr)
		fmt.Println("Len:", C.int(result.Len))
		defer C.free_result(handle, result)
	}
}
