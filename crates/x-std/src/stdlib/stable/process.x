pub fn exit(code: i32) -> void {
    intrinsic::process_exit(code);
}

pub fn abort() -> void {
    intrinsic::process_abort();
}
