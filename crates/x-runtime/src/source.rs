pub const ABI_HEADER: &str = include_str!("../include/x_runtime.h");

pub const HOSTED: &str = concat!(
    include_str!("../include/x_runtime.h"),
    "\n#define X_RUNTIME_AMALGAMATED 1\n",
    include_str!("../native/hosted.c"),
);

pub const FREESTANDING: &str = include_str!("../native/freestanding.c");

pub const WINDOWS_UCRT: &str = include_str!("../native/windows_ucrt.c");

pub(super) const HOSTED_WITH_PARALLEL: &str = concat!(
    include_str!("../include/x_runtime.h"),
    "\n#define X_RUNTIME_AMALGAMATED 1\n",
    include_str!("../native/parallel.c"),
    "\n",
    include_str!("../native/hosted.c"),
);
