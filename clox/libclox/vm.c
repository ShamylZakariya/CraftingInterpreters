#include <stdio.h>

#include "common.h"
#include "debug.h"
#include "memory.h"
#include "vm.h"

//-----------------------------------------------------------------------------
VM vm;

static void resetStack()
{
    FREE_ARRAY(Value, vm.stack, vm.stackCapacity);
    vm.stackTop = vm.stack;
    vm.stackCapacity = 0;
}

static InterpretResult run()
{
#define READ_BYTE() (*vm.ip++)
#define READ_CONSTANT() (vm.chunk->constants.values[READ_BYTE()])
#define BINARY_OP(op) \
    do{ \
        double b = pop(); \
        double a = pop(); \
        push(a op b); \
    } while(false)

    for (;;) {
#ifdef DEBUG_TRACE_EXECUTION
        printf("          ");
        for (Value* slot = vm.stack; slot < vm.stackTop; slot++) {
            printf("[ ");
            printValue(*slot);
            printf(" ]");
        }
        printf("\n");

        disassembleInstruction(vm.chunk, (int)(vm.ip - vm.chunk->code));
#endif

        uint8_t instruction;
        switch (instruction = READ_BYTE()) {
        case OP_CONSTANT: {
            Value constant = READ_CONSTANT();
            push(constant);
            break;
        }
        case OP_CONSTANT_LONG: {
            int a = READ_BYTE();
            int b = READ_BYTE();
            int c = READ_BYTE();
            Value constant = vm.chunk->constants.values[a << 16 | b << 8 | c];
            push(constant);
            break;
        }
        case OP_ADD: BINARY_OP(+); break;
        case OP_SUBTRACT: BINARY_OP(-); break;
        case OP_MULTIPLY: BINARY_OP(*); break;
        case OP_DIVIDE: BINARY_OP(/); break;
        case OP_NEGATE:
            push(-pop());
            break;
        case OP_RETURN: {
            printValue(pop());
            printf("\n");
            return INTERPRET_OK;
        }
        }
    }

#undef READ_BYTE
#undef READ_CONSTANT
#undef BINARY_OP
}

//-----------------------------------------------------------------------------

void initVM()
{
    vm.stack = NULL;
    vm.stackCapacity = 0;
    vm.stackTop = vm.stack;
    resetStack();
}

void freeVM()
{
    resetStack();
}

void push(Value value)
{
    // Capter 15 challenge 3: Growable stack
    int count = (int)(vm.stackTop - vm.stack);
    if (count == vm.stackCapacity) {
        int oldCapacity = vm.stackCapacity;
        vm.stackCapacity = GROW_CAPACITY(oldCapacity);
        vm.stack = GROW_ARRAY(vm.stack, Value, oldCapacity, vm.stackCapacity);
        vm.stackTop = vm.stack + count;
    }
    *vm.stackTop = value;
    vm.stackTop++;
}

Value pop()
{
    vm.stackTop--;
    return *vm.stackTop;
}

InterpretResult interpret(Chunk* chunk)
{
    vm.chunk = chunk;
    vm.ip = vm.chunk->code;
    return run();
}
