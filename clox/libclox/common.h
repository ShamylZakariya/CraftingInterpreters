#ifndef clox_common_h
#define clox_common_h

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

// #define DEBUG_PRINT_CODE
// #define DEBUG_TRACE_EXECUTION
// #define DEBUG_STRESS_GC
// #define DEBUG_LOG_GC

// enables 64-bit value types instead of wider 128 bit unions
// by using 48-bit pointers in the mantissa of a NaN double
#define NAN_BOXING

#define UINT8_COUNT (UINT8_MAX + 1)

#endif