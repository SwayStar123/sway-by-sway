library b256_ops;

use ::panic::panic;
use ::chain::log_u64;

pub trait Shiftable {
    fn lsh(self, other: Self) -> Self;
    fn rsh(self, other: Self) -> Self;
}

impl Shiftable for u64 {
    fn lsh(self, other: Self) -> Self {
        asm(r1: self, r2: other, r3) {
            sll r3 r1 r2;
            r3: u64
        }
    }
    fn rsh(self, other: Self) -> Self {
        asm(r1: self, r2: other, r3) {
            srl r3 r1 r2;
            r3: u64
        }
    }
}

impl u64 {
    fn binary_and(self, other: Self) -> Self {
        asm(r1: self, r2: other, r3) {
            and r3 r1 r2;
            r3: u64
        }
    }

    fn binary_or(self, other: Self) -> Self {
        asm(r1: self, r2: other, r3) {
            or r3 r1 r2;
            r3: u64
        }
    }

    fn binary_xor(self, other: Self) -> Self {
        asm(r1: self, r2: other, r3) {
            xor r3 r1 r2;
            r3: u64
        }
    }
}

impl b256 {
    pub fn and_b256(val: self, other: Self) -> Self {
        let (value_word_1, value_word_2, value_word_3, value_word_4) = decompose(val);
        let (other_word_1, other_word_2, other_word_3, other_word_4) = decompose(other);
        let word_1 = value_word_1.binary_and(other_word_1);
        let word_2 = value_word_2.binary_and(other_word_2);
        let word_3 = value_word_3.binary_and(other_word_3);
        let word_4 = value_word_4.binary_and(other_word_4);
        let rebuilt = compose(word_1, word_2, word_3, word_4);
        rebuilt
    }

    pub fn or_b256(val: self, other: Self) -> Self {
        let (value_word_1, value_word_2, value_word_3, value_word_4) = decompose(val);
        let (other_word_1, other_word_2, other_word_3, other_word_4) = decompose(other);
        let word_1 = value_word_1.binary_or(other_word_1);
        let word_2 = value_word_2.binary_or(other_word_2);
        let word_3 = value_word_3.binary_or(other_word_3);
        let word_4 = value_word_4.binary_or(other_word_4);
        let rebuilt = compose(word_1, word_2, word_3, word_4);
        rebuilt
    }

    pub fn xor_b256(val: self, other: Self) -> Self {
        let (value_word_1, value_word_2, value_word_3, value_word_4) = decompose(val);
        let (other_word_1, other_word_2, other_word_3, other_word_4) = decompose(other);
        let word_1 = value_word_1.binary_xor(other_word_1);
        let word_2 = value_word_2.binary_xor(other_word_2);
        let word_3 = value_word_3.binary_xor(other_word_3);
        let word_4 = value_word_4.binary_xor(other_word_4);
        let rebuilt = compose(word_1, word_2, word_3, word_4);
        rebuilt
    }

    pub fn lsh_b256(val: self, n: u64) -> Self {
        let (w1, w2, w3, w4) = decompose(val);

        let (word_1, overflow_1) = shift_left_with_overflow(w1, n);
        let (word_2, overflow_2) = shift_left_with_overflow(w2, n);
        let (word_3, overflow_3) = shift_left_with_overflow(w3, n);
        let (word_4, overflow_4) = shift_left_with_overflow(w4, n);

        // Use ADD or binary_or, whichever is cheaper
        let w1_shifted = if overflow_2 != 0 {
            word_1.binary_or(overflow_2)
        } else {
            word_1
        };
        let w2_shifted = if overflow_3 != 0 {
            word_2.binary_or(overflow_3)
        } else {
            word_2
        };
        let w3_shifted = if overflow_4 != 0 {
            word_3.binary_or(overflow_4)
        } else {
            word_3
        };
        let w4_shifted = word_4.lsh(n);

        compose(w1_shifted, w2_shifted, w3_shifted, w4_shifted)
    }

    pub fn rsh_b256(val: self, n: u64) -> Self {
        let (w1, w2, w3, w4) = decompose(val);

        let (word_1, overflow_1) = shift_right_with_overflow(w1, n);
        let (word_2, overflow_2) = shift_right_with_overflow(w2, n);
        let (word_3, overflow_3) = shift_right_with_overflow(w3, n);
        let (word_4, overflow_4) = shift_right_with_overflow(w4, n);

        // Use ADD or binary_or, whichever is cheaper
        let w4_shifted = if overflow_3 != 0 {
            word_4.binary_or(overflow_3)
        } else {
            word_4
        };
        let w3_shifted = if overflow_2 != 0 {
            word_3.binary_or(overflow_2)
        } else {
            word_3
        };
        let w2_shifted = if overflow_1 != 0 {
            word_2.binary_or(overflow_1)
        } else {
            word_2
        };
        let w1_shifted = word_1.rsh(n);

        compose(w1_shifted, w2_shifted, w3_shifted, w4_shifted)
    }

}

// Extract a singe word from a b256 value using a specified offset.
pub fn get_word_from_b256(val: b256, offset: u64) -> u64 {
    let mut empty: u64 = 0;
    asm(r1: val, offset: offset, r2,  res: empty) {
        add r2 r1 offset;
        lw res r2 i0;
        res: u64
    }
}

// Get 4 words from a single b256 value.
pub fn decompose(val: b256) -> (u64, u64, u64, u64) {
    let w1 = get_word_from_b256(val, 0);
    let w2 = get_word_from_b256(val, 8);
    let w3 = get_word_from_b256(val, 16);
    let w4 = get_word_from_b256(val, 24);
    (w1, w2, w3, w4)
}

// Build a single b256 value from 4 words.
pub fn compose(word_1: u64, word_2: u64, word_3: u64, word_4: u64) -> b256 {
    let res: b256 = 0x0000000000000000000000000000000000000000000000000000000000000000;
    asm(w1: word_1, w2: word_2, w3: word_3, w4: word_4, result: res) {
        sw result w1 i0;
        sw result w2 i1;
        sw result w3 i2;
        sw result w4 i3;
        result: b256
    }
}

const FLAG = 2;

fn shift_left_with_overflow(word: u64, shift_amount: u64) -> (u64, u64) {
    let mut output = (0, 0);
    let right_shift_amount = 64 - shift_amount;
    let (shifted, overflow) = asm(out: output, r1: word, r2: shift_amount, r3, r4, r5: FLAG, r6: right_shift_amount) {
       flag r5;        // set flag to allow overflow without panic
       srl r3 r1 r6;   // shift right to get overflow, put result in r3
       sll r4 r1 r2;   // shift left, put result in r4
       sw out r4 i0;   // store word at r4 in output
       sw out r3 i1;   // store word at r3 in output + 1 word offset
       out: (u64, u64) // return both values
    };

    (shifted, overflow)
}

fn shift_right_with_overflow(word: u64, shift_amount: u64) -> (u64, u64) {
    let mut output = (0, 0);
    let left_shift_amount = 64 - shift_amount;
    let (shifted, overflow) = asm(out: output, r1: word, r2: shift_amount, r3, r4, r5: FLAG, r6: left_shift_amount) {
       flag r5;        // set flag to allow overflow without panic
       sll r3 r1 r6;   // shift left to get overflow, put result in r3
       srl r4 r1 r2;   // shift right, put result in r4
       sw out r4 i0;   // store word at r4 in output
       sw out r3 i1;   // store word at r3 in output + 1 word offset
       out: (u64, u64) // return both values
    };

    (shifted, overflow)
}
