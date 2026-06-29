#include <stddef.h>

__attribute__((weak)) void *memset(void *destination, int value, size_t count) {
    unsigned char *destination_bytes = (unsigned char *)destination;
    for (size_t offset = 0; offset < count; ++offset) {
        destination_bytes[offset] = (unsigned char)value;
    }
    return destination;
}

__attribute__((weak)) void *memcpy(void *destination, const void *source, size_t count) {
    unsigned char *destination_bytes = (unsigned char *)destination;
    const unsigned char *source_bytes = (const unsigned char *)source;
    for (size_t offset = 0; offset < count; ++offset) {
        destination_bytes[offset] = source_bytes[offset];
    }
    return destination;
}

__attribute__((weak)) void *memmove(void *destination, const void *source, size_t count) {
    unsigned char *destination_bytes = (unsigned char *)destination;
    const unsigned char *source_bytes = (const unsigned char *)source;
    if ((size_t)destination_bytes < (size_t)source_bytes) {
        for (size_t offset = 0; offset < count; ++offset) {
            destination_bytes[offset] = source_bytes[offset];
        }
    } else {
        while (count != 0) {
            --count;
            destination_bytes[count] = source_bytes[count];
        }
    }
    return destination;
}
