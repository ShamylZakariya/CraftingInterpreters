#include <stdio.h>

#include <libclox/chunk.h>
#include <libclox/common.h>
#include <libclox/debug.h>

int main(int argc, const char* argv[])
{

    Chunk chunk;
    initChunk(&chunk);

    for (int i = 0; i < 300; i++) {
      Value val = (i + 30.0) * 14.567;
      writeConstant(&chunk, val, i);
    }
    writeChunk(&chunk, OP_RETURN, 123);

    disassembleChunk(&chunk, "test chunk");

    freeChunk(&chunk);

    return 0;
}