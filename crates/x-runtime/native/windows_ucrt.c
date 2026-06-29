#ifndef WIN32_LEAN_AND_MEAN
#define WIN32_LEAN_AND_MEAN
#endif
#ifndef NOUSER
#define NOUSER
#endif
#ifndef NOGDI
#define NOGDI
#endif
#include <stdarg.h>
#include <stdio.h>
#include <stdlib.h>
#include <windows.h>

extern FILE *__acrt_iob_func(unsigned index);
extern int __stdio_common_vfprintf(unsigned long long options, FILE *stream, const char *format,
                                   void *locale, va_list arguments);

#undef stdin
#undef stdout
#undef stderr
#define stdin (__acrt_iob_func(0))
#define stdout (__acrt_iob_func(1))
#define stderr (__acrt_iob_func(2))

// TinyCC's va_list must match the Windows x64 ABI.
int printf(const char *format, ...) {
    va_list arguments;
    va_start(arguments, format);
    int result = __stdio_common_vfprintf(0, stdout, format, NULL, arguments);
    va_end(arguments);
    return result;
}

extern int _seh_filter_exe(unsigned long code, void *exception);
extern void _set_app_type(int type);
extern int _configure_narrow_argv(int mode);
extern int _initialize_narrow_environment(void);
extern int *__p___argc(void);
extern char ***__p___argv(void);
extern char ***__p__environ(void);
extern void (*__init_array_start[])(int, char **, char **);
extern void (*__init_array_end[])(int, char **, char **);
extern void (*__fini_array_start[])(void);
extern void (*__fini_array_end[])(void);
extern int main(int argument_count, char **argument_values, char **environment);

static LONG WINAPI x_tinycc_handle_exception(EXCEPTION_POINTERS *exception) {
    return _seh_filter_exe(exception->ExceptionRecord->ExceptionCode, exception);
}

enum {
    X_TINYCC_CONSOLE_APP = 1,
    X_TINYCC_UNEXPANDED_ARGV = 1,
    X_TINYCC_STARTUP_FAILURE = 255,
};

void _start(void) {
    SetUnhandledExceptionFilter(x_tinycc_handle_exception);
    _set_app_type(X_TINYCC_CONSOLE_APP);
    if (_configure_narrow_argv(X_TINYCC_UNEXPANDED_ARGV) || _initialize_narrow_environment()) {
        exit(X_TINYCC_STARTUP_FAILURE);
    }
    int argument_count = *__p___argc();
    char **argument_values = *__p___argv();
    char **environment = *__p__environ();
    for (int index = 0; &__init_array_start[index] != __init_array_end; ++index) {
        __init_array_start[index](argument_count, argument_values, environment);
    }
    int exit_code = main(argument_count, argument_values, environment);
    for (int index = 0; &__fini_array_end[index] != __fini_array_start;) {
        __fini_array_end[--index]();
    }
    exit(exit_code);
}
