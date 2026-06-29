#ifndef X_RUNTIME_H
#define X_RUNTIME_H

#include <stddef.h>
#include <stdint.h>

#define X_RUNTIME_ABI_VERSION 2u

#if defined(_WIN32)
#define X_NORETURN __declspec(noreturn)
#else
#define X_NORETURN _Noreturn
#endif

typedef struct x_throws_i32 {
    /* Storage layout, not a C return ABI. */
    uint8_t succeeded;
    int32_t value;
    const char *error;
} x_throws_i32;

typedef int32_t (*x_parallel_worker_i32)(int32_t index);
typedef int32_t (*x_parallel_worker_context_i32)(int32_t index, const void *context);

uint32_t x_runtime_abi_version(void);
void *x_runtime_alloc(size_t size);
void *x_runtime_realloc(void *pointer, size_t size);
void x_runtime_free(void *pointer);
size_t x_runtime_string_len(const char *string);
/* Writes exactly the descriptor byte length, including embedded NUL bytes. */
void x_runtime_write_stdout(const char *data, size_t length);
int32_t x_runtime_exit_code(int32_t code);
X_NORETURN void x_runtime_process_exit(int32_t code);
X_NORETURN void x_runtime_panic(const char *message);
/* Callbacks finish before this function returns. */
void dispatch_parallel_for(int32_t start, int32_t end, x_parallel_worker_i32 worker);
/* Context stays read-only and borrowed until all callbacks finish. */
void dispatch_parallel_for_context(int32_t start, int32_t end, x_parallel_worker_context_i32 worker,
                                   const void *context);

#endif
