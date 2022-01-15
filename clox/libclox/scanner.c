#include <stdio.h>
#include <string.h>

#include "common.h"
#include "scanner.h"

//-------------------------------------------------------------------

typedef struct {
    const char* start; // beginning of current lexeme
    const char* current; // current character
    int line;
} Scanner;

Scanner scanner;

//-------------------------------------------------------------------

void initScanner(const char* source)
{
    scanner.start = source;
    scanner.current = source;
    scanner.line = 1;
}