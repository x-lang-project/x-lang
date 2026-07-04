// String lengths are UTF-8 byte counts.

pub pure fn len(value: str) -> usize {
    return value.len();
}

pub pure fn is_empty(value: str) -> bool {
    return value.len() == 0usize;
}
