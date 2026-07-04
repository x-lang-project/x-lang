
pub pure fn len<A>(values: &A) -> usize {
    return values.len();
}

pub pure fn is_empty<A>(values: &A) -> bool {
    return values.len() == 0usize;
}

pub pure fn contains<A, T>(values: &A, target: T) -> bool {
    let mut index: usize = 0usize;
    while index < values.len() {
        if values[index] == target {
            return true;
        }
        index = index + 1usize;
    }
    return false;
}

pub pure fn count<A, T>(values: &A, target: T) -> usize {
    let mut index: usize = 0usize;
    let mut result: usize = 0usize;
    while index < values.len() {
        if values[index] == target {
            result = result + 1usize;
        }
        index = index + 1usize;
    }
    return result;
}

pub pure fn find<A, T>(values: &A, target: T) -> usize {
    let mut index: usize = 0usize;
    while index < values.len() {
        if values[index] == target {
            return index;
        }
        index = index + 1usize;
    }
    return values.len();
}
