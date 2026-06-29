#ifndef X_RUNTIME_AMALGAMATED
#include "x_runtime.h"
#endif

#include <omp.h>
#include <stdbool.h>

enum { X_PARALLEL_SERIAL_THRESHOLD = 64 };

static int x_runtime_parallel_thread_count(int64_t count) {
    const int available = omp_get_max_threads();
    const int threads = count < available ? (int)count : available;
    return threads < 1 ? 1 : threads;
}

/* Preserve nested-team behaviour inside an existing OpenMP team. */
static bool x_runtime_parallel_runs_serially(int64_t count, int threads) {
    return threads == 1 && count <= X_PARALLEL_SERIAL_THRESHOLD && omp_get_level() == 0;
}

void dispatch_parallel_for(int32_t start, int32_t end, x_parallel_worker_i32 worker) {
    if (end <= start || worker == NULL) {
        return;
    }
    const int64_t count = (int64_t)end - (int64_t)start;
    const int threads = x_runtime_parallel_thread_count(count);
    if (x_runtime_parallel_runs_serially(count, threads)) {
        for (int64_t index = (int64_t)start; index < (int64_t)end; ++index) {
            (void)worker((int32_t)index);
        }
        return;
    }

#pragma omp parallel for num_threads(threads) schedule(static)
    for (int64_t index = (int64_t)start; index < (int64_t)end; ++index) {
        (void)worker((int32_t)index);
    }
}

void dispatch_parallel_for_context(int32_t start, int32_t end, x_parallel_worker_context_i32 worker,
                                   const void *context) {
    if (end <= start || worker == NULL || context == NULL) {
        return;
    }
    const int64_t count = (int64_t)end - (int64_t)start;
    const int threads = x_runtime_parallel_thread_count(count);
    if (x_runtime_parallel_runs_serially(count, threads)) {
        for (int64_t index = (int64_t)start; index < (int64_t)end; ++index) {
            (void)worker((int32_t)index, context);
        }
        return;
    }
#pragma omp parallel for num_threads(threads) schedule(static)
    for (int64_t index = (int64_t)start; index < (int64_t)end; ++index) {
        (void)worker((int32_t)index, context);
    }
}
