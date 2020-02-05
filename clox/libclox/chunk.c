#include <stdlib.h>

#include "chunk.h"
#include "memory.h"

void initChunk(Chunk* chunk)
{
    chunk->count = 0;
    chunk->capacity = 0;
    chunk->code = NULL;
    chunk->lines = NULL;
    initValueArray(&chunk->constants);
}

void freeChunk(Chunk* chunk)
{
    FREE_ARRAY(uint8_t, chunk->code, chunk->capacity);
    FREE_ARRAY(int, chunk->lines, chunk->capacity);
    freeValueArray(&chunk->constants);
    initChunk(chunk);
}

void writeChunk(Chunk* chunk, uint8_t byte, int line)
{
    if (chunk->capacity < chunk->count + 1) {
        int oldCapacity = chunk->capacity;
        chunk->capacity = GROW_CAPACITY(oldCapacity);
        chunk->code = GROW_ARRAY(chunk->code, uint8_t, oldCapacity, chunk->capacity);
        chunk->lines = GROW_ARRAY(chunk->lines, int, oldCapacity, chunk->capacity);
    }

    chunk->code[chunk->count] = byte;
    chunk->lines[chunk->count] = line;
    chunk->count++;
}

int addConstant(Chunk* chunk, Value value)
{
    writeValueArray(&chunk->constants, value);
    return chunk->constants.count - 1;
}

void writeConstant(Chunk* chunk, Value value, int line)
{
    // chapter 14: challenge 2
    int idx = addConstant(chunk, value);
    writeChunk(chunk, OP_CONSTANT_LONG, line);

    // store the constant's index as 3 bytes
    uint8_t a = (idx & 0x00FF0000) >> 16;
    uint8_t b = (idx & 0x0000FF00) >> 8;
    uint8_t c = (idx& 0x000000FF) >> 0;

    writeChunk(chunk, a, line);
    writeChunk(chunk, b, line);
    writeChunk(chunk, c, line);
}
