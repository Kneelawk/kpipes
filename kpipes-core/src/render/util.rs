pub fn least_power_of_2_greater(x: u64) -> u64 {
    if x < 1 {
        return 0;
    }

    let mut x = x;
    x -= 1;
    x |= x >> 1;
    x |= x >> 2;
    x |= x >> 4;
    x |= x >> 8;
    x |= x >> 16;
    x |= x >> 32;
    x + 1
}

#[cfg(test)]
mod tests {
    use crate::render::util::least_power_of_2_greater;

    #[test]
    fn test_least_power_of_2_greater() {
        assert_eq!(least_power_of_2_greater(5), 8);
        assert_eq!(least_power_of_2_greater(7), 8);
        assert_eq!(least_power_of_2_greater(8), 8);
        assert_eq!(least_power_of_2_greater(33), 64);
        assert_eq!(least_power_of_2_greater(528), 1024);
        assert_eq!(least_power_of_2_greater(46021), 65536);
        assert_eq!(least_power_of_2_greater(821032), 1048576);
    }
}
