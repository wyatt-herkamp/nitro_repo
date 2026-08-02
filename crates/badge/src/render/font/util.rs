pub(crate) fn is_control_char(c: u32) -> bool {
    c <= 31 || c == 127
}
