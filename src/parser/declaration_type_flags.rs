use super::ScalarType;

pub(super) const SAW_TYPE: u8 = 1;
pub(super) const SAW_STORAGE_CLASS: u8 = 2;
pub(super) const SAW_POINTER: u8 = 4;
pub(super) const SAW_BOOL: u8 = 8;
pub(super) const SAW_DOUBLE: u8 = 16;
pub(super) const SAW_COMPLEX: u8 = 32;
pub(super) const SAW_FLOAT: u8 = 64;

pub(super) const fn declaration_type_from_flags(
    flags: u8,
    long_count: usize,
    index: usize,
) -> Option<(ScalarType, usize)> {
    if !has_flag(flags, SAW_TYPE) {
        if has_flag(flags, SAW_STORAGE_CLASS) {
            return Some((ScalarType::Int, index));
        }
        return None;
    }
    if has_flag(flags, SAW_POINTER) {
        Some((ScalarType::Pointer, index))
    } else if has_flag(flags, SAW_COMPLEX) {
        Some((complex_declaration_type(flags, long_count), index))
    } else if has_flag(flags, SAW_FLOAT) {
        None
    } else if has_flag(flags, SAW_BOOL) {
        Some((ScalarType::Bool, index))
    } else if has_flag(flags, SAW_DOUBLE) && long_count == 0 {
        Some((ScalarType::Double, index))
    } else if has_flag(flags, SAW_DOUBLE) {
        Some((ScalarType::LongDouble, index))
    } else if long_count == 0 {
        Some((ScalarType::Int, index))
    } else {
        Some((ScalarType::LongLong, index))
    }
}

const fn complex_declaration_type(flags: u8, long_count: usize) -> ScalarType {
    if has_flag(flags, SAW_FLOAT) {
        ScalarType::ComplexFloat
    } else if long_count == 0 {
        ScalarType::ComplexDouble
    } else {
        ScalarType::ComplexLongDouble
    }
}

pub(super) const fn bool_flag(value: bool, flag: u8) -> u8 {
    if value { flag } else { 0 }
}

const fn has_flag(flags: u8, flag: u8) -> bool {
    flags & flag != 0
}
