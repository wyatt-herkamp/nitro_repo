pub fn round_up_to_odd(val: usize) -> usize {
    if val.is_multiple_of(2) { val + 1 } else { val }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_test() {
        assert_eq!(round_up_to_odd(10), 11);
        assert_eq!(round_up_to_odd(11), 11);
    }
}
