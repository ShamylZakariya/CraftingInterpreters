#include <stdlib.h>
#include <string.h>

#include "memory.h"
#include "object.h"
#include "table.h"
#include "value.h"

//-----------------------------------------------------------------------------

#define TABLE_MAX_LOAD 0.75

static Entry* findEntry(Entry* entries, int capacity, ObjString* key)
{
    uint32_t index = key->hash % capacity;
    Entry* tombstone = NULL;

    for (;;) {
        Entry* entry = &entries[index];

        if (entry->key == NULL) {
            if (IS_NIL(entry->value)) {
                // we found an empty spot; if we've traversed through
                // any tombstones on the way, reuse the first one;
                // otherwise, use this first empty space
                return tombstone != NULL ? tombstone : entry;
            } else {
                // we've landed on a tombstone; if it's the first
                // tombstone in our traversal, record it for
                // use later
                if (tombstone == NULL) {
                    tombstone = entry;
                }
            }
        } else if (entry->key == key) {
            // we found the key
            return entry;
        }

        index = (index + 1) % capacity;
    }
}

static void adjustCapacity(Table* table, int capacity)
{
    Entry* entries = ALLOCATE(Entry, capacity);
    for (int i = 0; i < capacity; i++) {
        entries[i].key = NULL;
        entries[i].value = NIL_VAL;
    }

    // copy over previous entries -- reset
    // count because we're not copying over tombstones
    table->count = 0;
    for (int i = 0; i < table->capacity; i++) {
        Entry* entry = &table->entries[i];
        if (entry->key == NULL) {
            continue;
        }

        Entry* dest = findEntry(entries, capacity, entry->key);
        dest->key = entry->key;
        dest->value = entry->value;
        table->count++;
    }

    FREE_ARRAY(Entry, table->entries, table->capacity);
    table->entries = entries;
    table->capacity = capacity;
}

//-----------------------------------------------------------------------------

void initTable(Table* table)
{
    table->count = 0;
    table->capacity = 0;
    table->entries = NULL;
}

void freeTable(Table* table)
{
    FREE_ARRAY(Entry, table->entries, table->capacity);
    initTable(table);
}

bool tableGet(Table* table, ObjString* key, Value* value)
{
    if (table->count == 0)
        return false;

    Entry* entry = findEntry(table->entries, table->capacity, key);
    if (entry->key == NULL)
        return false;

    *value = entry->value;
    return true;
}

bool tableSet(Table* table, ObjString* key, Value value)
{
    if (table->count + 1 > table->capacity * TABLE_MAX_LOAD) {
        int capacity = GROW_CAPACITY(table->capacity);
        adjustCapacity(table, capacity);
    }

    Entry* entry = findEntry(table->entries, table->capacity, key);

    bool isNewKey = entry->key == NULL;
    if (isNewKey && IS_NIL(entry->value)) {
        // this is a new (not-reusing-tombstone) key
        table->count++;
    }

    entry->key = key;
    entry->value = value;
    return isNewKey;
}

bool tableDelete(Table* table, ObjString* key)
{
    if (table->count == 0)
        return false;

    // Find it
    Entry* entry = findEntry(table->entries, table->capacity, key);
    if (entry->key == NULL)
        return false;

    // place tombstone - note we don't decrement count
    // because if we did, we could end up with a scenario where
    // the storage is full of tombstones and has no empty spots,
    // which would result in an infinite loop in findEntry
    entry->key = NULL;
    entry->value = BOOL_VAL(true);

    return true;
}

void tableAddAll(Table* from, Table* to)
{
    for (int i = 0; i < from->capacity; i++) {
        Entry* entry = &from->entries[i];
        if (entry->key != NULL) {
            tableSet(to, entry->key, entry->value);
        }
    }
}

ObjString* tableFindString(Table* table, const char* chars, int length, uint32_t hash)
{
    if (table->count == 0)
        return NULL;

    uint32_t index = hash % table->capacity;

    for (;;) {
        Entry* entry = &table->entries[index];

        if (entry->key == NULL) {
            // stop on empty non-tombstone entry
            if (IS_NIL(entry->value))
                return NULL;
        } else if ((entry->key->length == length)
            && (entry->key->hash == hash)
            && (memcmp(entry->key->chars, chars, length) == 0)) {
            // found it
            return entry->key;
        }

        index = (index + 1) % table->capacity;
    }
}
