#include <stdlib.h>

#include "memory.h"
#include "vm.h"

static void freeObject(Obj* object)
{
    switch (object->type) {
    case OBJ_CLOSURE: {
        ObjClosure* closure = (ObjClosure*) object;
        FREE_ARRAY(ObjUpvalue*, closure->upvalues, closure->upvalueCount);
        // free only the closure, not the function. Other closures may be using it.
        FREE(ObjClosure, object);
        break;
    }
    case OBJ_FUNCTION: {
        ObjFunction* function = (ObjFunction*)object;
        freeChunk(&function->chunk);
        FREE(ObjFunction, object);
        // function->name will be handled by GC
        break;
    }
    case OBJ_NATIVE:
        FREE(ObjNative, object);
        break;
    case OBJ_STRING: {
        ObjString* string = (ObjString*)object;
        FREE_ARRAY(char, string->chars, string->length + 1);
        FREE(ObjString, object);
        break;
    }
    case OBJ_UPVALUE:
        // Upvalue does not own the closed over variable
        FREE(ObjUpvalue, object);
        break;
    }
}

//-------------------------------------------------------------------

void* reallocate(void* pointer, size_t oldSize, size_t newSize)
{
    if (newSize == 0) {
        free(pointer);
        return NULL;
    }

    void* result = realloc(pointer, newSize);
    if (result == NULL) {
        exit(1);
    }
    return result;
}

void freeObjects()
{
    Obj* object = vm.objects;
    while (object != NULL) {
        Obj* next = object->next;
        freeObject(object);
        object = next;
    }
}