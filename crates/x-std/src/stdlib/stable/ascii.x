
pub pure fn is_ascii(value: char) -> bool {
    return (value as u8) <= 127u8;
}

pub pure fn is_control(value: char) -> bool {
    return (value as u8) < 32u8 || (value as u8) == 127u8;
}

pub pure fn is_whitespace(value: char) -> bool {
    // Byte whitespace excludes vertical tab.
    return value == ' ' || value == '\t' || value == '\n' ||
        (value as u8) == 12u8 || value == '\r';
}

pub pure fn is_uppercase(value: char) -> bool {
    return value >= 'A' && value <= 'Z';
}

pub pure fn is_lowercase(value: char) -> bool {
    return value >= 'a' && value <= 'z';
}

pub pure fn is_alphabetic(value: char) -> bool {
    return ascii::is_uppercase(value) || ascii::is_lowercase(value);
}

pub pure fn is_digit(value: char) -> bool {
    return value >= '0' && value <= '9';
}

pub pure fn is_alphanumeric(value: char) -> bool {
    return ascii::is_alphabetic(value) || ascii::is_digit(value);
}

pub pure fn is_hex_digit(value: char) -> bool {
    return ascii::is_digit(value) || (value >= 'A' && value <= 'F') ||
        (value >= 'a' && value <= 'f');
}

pub pure fn is_graphic(value: char) -> bool {
    return (value as u8) >= 33u8 && (value as u8) <= 126u8;
}

pub pure fn is_printable(value: char) -> bool {
    return value == ' ' || ascii::is_graphic(value);
}

pub pure fn is_punctuation(value: char) -> bool {
    return ascii::is_graphic(value) && !ascii::is_alphanumeric(value);
}

pub pure fn to_lowercase(value: char) -> char {
    if ascii::is_uppercase(value) {
        return ((value as u8) + 32u8) as char;
    }
    return value;
}

pub pure fn to_uppercase(value: char) -> char {
    if ascii::is_lowercase(value) {
        return ((value as u8) - 32u8) as char;
    }
    return value;
}

pub pure fn eq_ignore_case(left: char, right: char) -> bool {
    return ascii::to_lowercase(left) == ascii::to_lowercase(right);
}

pub pure fn digit_value(value: char) -> i32 {
    if ascii::is_digit(value) {
        return (value as i32) - ('0' as i32);
    }
    return -1;
}

pub pure fn hex_value(value: char) -> i32 {
    if ascii::is_digit(value) {
        return ascii::digit_value(value);
    }
    if value >= 'A' && value <= 'F' {
        return (value as i32) - ('A' as i32) + 10;
    }
    if value >= 'a' && value <= 'f' {
        return (value as i32) - ('a' as i32) + 10;
    }
    return -1;
}
