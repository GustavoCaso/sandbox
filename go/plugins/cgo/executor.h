// From the Go documentation, it's recommended to include stdlib.h if we need
// to use C.free.
#include <stdlib.h>
#include <stdbool.h>

typedef struct
{
  char *Char;
  int Len;
} Result;

extern void *open_library(char *library);
extern void close_library(void *handle);
extern Result *run_check(void *handle);
extern void free_result(void *handle, Result *result);