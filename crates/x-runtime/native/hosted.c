#ifndef X_RUNTIME_AMALGAMATED
#include "x_runtime.h"
#endif

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#if defined(_WIN32)
#ifndef WIN32_LEAN_AND_MEAN
#define WIN32_LEAN_AND_MEAN
#endif
#ifndef NOUSER
#define NOUSER
#endif
#ifndef NOGDI
#define NOGDI
#endif
#include <windows.h>
int getpagesize(void) {
    SYSTEM_INFO info;
    GetSystemInfo(&info);
    return (int)info.dwPageSize;
}
#endif

uint32_t x_runtime_abi_version(void) {
    return X_RUNTIME_ABI_VERSION;
}

void *x_runtime_alloc(size_t size) {
    return malloc(size == 0 ? 1 : size);
}

void *x_runtime_realloc(void *pointer, size_t size) {
    return realloc(pointer, size == 0 ? 1 : size);
}

void x_runtime_free(void *pointer) {
    free(pointer);
}

size_t x_runtime_string_len(const char *string) {
    if (string == NULL) {
        return 0;
    }
    return strlen(string);
}

int32_t x_runtime_exit_code(int32_t code) {
    return code;
}

void x_runtime_write_stdout(const char *data, size_t length) {
    if (length != 0) {
        (void)fwrite(data, 1, length, stdout);
    }
}

X_NORETURN void x_runtime_process_exit(int32_t code) {
    fflush(NULL);
    exit(code);
}

X_NORETURN void x_runtime_panic(const char *message) {
    fputs("X panic: ", stderr);
    fputs(message == NULL ? "<no message>" : message, stderr);
    fputc('\n', stderr);
    fflush(stderr);
    abort();
}
