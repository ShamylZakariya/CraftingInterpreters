#ifndef clox_value_h
#define clox_value_h

#include "common.h"

// forward decls from object.h
typedef struct _Obj Obj;
typedef struct _ObjString ObjString;

typedef enum _ValueType {
    VAL_BOOL,
    VAL_NIL,
    VAL_NUMBER,
    VAL_OBJ
} ValueType;

typedef struct _Value {
    ValueType type;
    union {
        bool boolean;
        double number;
        Obj* obj;
    } as;
} Value;

#define IS_BOOL(value) ((value).type == VAL_BOOL)
#define IS_NIL(value) ((value).type == VAL_NIL)
#define IS_NUMBER(value) ((value).type == VAL_NUMBER)
#define IS_OBJ(value) ((value).type == VAL_OBJ)

#define AS_OBJ(value) ((value).as.obj)
#define AS_BOOL(value) ((value).as.boolean)
#define AS_NUMBER(value) ((value).as.number)

#define BOOL_VAL(value) ((Value) { VAL_BOOL, { .boolean = value } })
#define NIL_VAL ((Value) { VAL_NIL, { .number = 0 } })
#define NUMBER_VAL(value) ((Value) { VAL_NUMBER, { .number = value } })
#define OBJ_VAL(object) ((Value) { VAL_OBJ, { .obj = (Obj*)object } })

typedef struct _ValueArray {
    int capacity;
    int count;
    Value* values;
} ValueArray;

bool valuesEqual(Value a, Value b);
void initValueArray(ValueArray* array);
void writeValueArray(ValueArray* array, Value value);
void freeValueArray(ValueArray* array);
void printValue(Value value);

#endif