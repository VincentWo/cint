use std::{cmp, fmt};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Dynamic {
    val: u8,
    bits: u8,
}

impl From<Dynamic> for i8 {
    fn from(value: Dynamic) -> Self {
        u8::from(value.sign_extend(8)) as i8
    }
}
impl From<Dynamic> for u8 {
    fn from(value: Dynamic) -> Self {
        value.val
    }
}

impl fmt::Debug for Dynamic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{0:#0width$b}", self.val, width = self.bits as usize + 2)
    }
}

impl std::ops::Not for Dynamic {
    type Output = Self;

    fn not(self) -> Self::Output {
        Dynamic::truncate(!self.val, self.bits)
    }
}
impl std::ops::BitAnd for Dynamic {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Dynamic {
            val: self.val & rhs.val,
            bits: cmp::min(self.bits, rhs.bits),
        }
    }
}

impl std::ops::BitAnd<u8> for Dynamic {
    type Output = Dynamic;

    fn bitand(self, rhs: u8) -> Self::Output {
        Dynamic {
            val: self.val & rhs,
            bits: self.bits,
        }
    }
}

impl std::ops::BitOr for Dynamic {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Dynamic {
            val: self.val | rhs.val,
            bits: cmp::max(self.bits, rhs.bits),
        }
    }
}

impl std::ops::Sub for Dynamic {
    type Output = Dynamic;

    fn sub(self, rhs: Self) -> Self::Output {
        assert!(self.bits == rhs.bits);

        Dynamic::truncate(self.val.wrapping_sub(rhs.val), self.bits)
    }
}

impl Dynamic {
    pub fn new(val: u8, bits: u8) -> Self {
        assert!((1..=8).contains(&bits));
        assert!(8 - val.leading_zeros() as u8 <= bits);
        Self { val, bits }
    }
    pub fn truncate(val: u8, bits: u8) -> Self {
        Self::ones(bits) & val
    }
    pub fn ones(count: u8) -> Self {
        Dynamic::new(((1u16 << count) - 1) as u8, count)
    }

    pub fn sign_extend(self, new_bits: u8) -> Dynamic {
        assert!((1..=8).contains(&new_bits));
        assert!(self.bits <= new_bits);

        let sign_bit_mask = 1_u8 << (self.bits - 1);
        let new_val = (self.val ^ sign_bit_mask) - sign_bit_mask;

        Dynamic::truncate(new_val, new_bits)
    }
    pub fn zero_extend(self, new_bits: u8) -> Dynamic {
        assert!(self.bits <= new_bits);
        Self {
            val: self.val,
            bits: new_bits,
        }
    }

    pub fn concat(self, rhs: Dynamic) -> Dynamic {
        Dynamic::new((self.val << rhs.bits) | rhs.val, self.bits + rhs.bits)
    }

    pub fn bits(self) -> u8 {
        self.bits
    }

    pub fn highest_set_bit(self) -> u8 {
        self.val.ilog2() as u8
    }

    //          0b101
    // >> 1: 0b0_0010
    // << 2: 0b1_0100
    //   or: 0b1_0110
    //       0b0_0111
    //  and: 0b0_0110
    pub fn rotate_right(self, len: u8) -> Self {
        let len = (len % self.bits) as u32;
        let left_shift = (self.bits as u32 - len) % 8;
        let new_val = (self.val >> len) | (self.val << left_shift);

        Dynamic::truncate(new_val, self.bits)
    }
}

pub fn replicate(val: Dynamic, count: u8) -> u64 {
    let shift = val.bits();
    let mut val: u64 = u8::from(val).into();

    for _ in 1..count {
        val |= val << shift
    }

    val
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn highest_set_bit() {
        let tests = [
            0b0000_0001,
            0b0000_0010,
            0b0000_0101,
            0b0000_1010,
            0b0001_1001,
            0b0010_0000,
            0b0101_0010,
            0b1010_0011,
        ];

        for (highest_bit, val) in tests.iter().enumerate() {
            let val = Dynamic::new(*val, 8);
            assert_eq!(highest_bit as u8, val.highest_set_bit());
        }
    }
    #[test]
    fn replicate() {
        let tests = [
            (0b1010, 3, 0b1010_1010_1010),
            (0b0110, 2, 0b0110_0110),
            (0b0101, 1, 0b0101),
            (0b1001, 4, 0b1001_1001_1001_1001),
        ];

        for (val, reps, res) in tests {
            let val = Dynamic::new(val, 4);

            let should_res = super::replicate(val, reps);

            assert!(should_res == res, "{should_res:#b} != {res:#b}");
        }
    }

    #[test]
    fn rotate_right() {
        let tests = [
            (3, 0, 0b101, 0b101),
            (3, 1, 0b101, 0b110),
            (4, 2, 0b1001, 0b0110),
            (5, 3, 0b11001, 0b00111),
            (8, 8, 0b1001_0110, 0b1001_0110),
        ];

        for (bits, rotate, unrotated, rotated) in tests {
            let unrotated = Dynamic::new(unrotated, bits);
            let rotated = Dynamic::new(rotated, bits);

            assert_eq!(
                rotated,
                unrotated.rotate_right(rotate),
                "Rotating {unrotated:?} by {rotate} bits should have resulted into left, but got right",
            );
        }
    }

    #[test]
    fn succeeding_new() {
        for bits in 1u8..=8 {
            for x in 0..2u16.pow(bits.into()) {
                Dynamic::new(x.try_into().unwrap(), bits);
            }
        }
    }

    #[test]
    fn failing_new() {
        let tests = [
            (0, 0),
            (1, 0b00000010),
            (2, 0b00000100),
            (3, 0b00001000),
            (4, 0b00010000),
            (5, 0b00100000),
            (6, 0b01000000),
            (7, 0b10000000),
            (9, 0),
        ];

        for (bits, val) in tests {
            let err = std::panic::catch_unwind(|| Dynamic::new(val, bits));

            err.unwrap_err();
        }
    }

    #[test]
    fn truncate_does_not_erase_too_much() {
        for bits in 1..=8 {
            let all_bits_set: u8 = Dynamic::ones(bits).into();
            for val in 0..=all_bits_set {
                let truncated: u8 = Dynamic::truncate(val, bits).into();
                assert_eq!(truncated, val);
            }
        }
    }

    #[test]
    fn truncate_erases_enough() {
        for bits in 1..8 {
            let all_bits_set: u8 = Dynamic::ones(bits).into();
            for val in all_bits_set..u8::MAX {
                let truncated: u8 = Dynamic::truncate(val, bits).into();
                assert_eq!(truncated >> bits, 0);
            }
        }
    }
}
