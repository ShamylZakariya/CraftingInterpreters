#!/bin/sh
find ./ \( -iname "*.h" -or -iname "*.c" \) | xargs clang-format -i -style=file
find ./libclox \( -iname "*.h" -or -iname "*.c" \) | xargs clang-format -i -style=file
