// Uses as u128 as a small array of len 8 where each 16bits is one "index"
#[derive(Debug, Clone)]
pub struct U128Arr {
    pub v: u128,
}
#[allow(dead_code)]
impl U128Arr {
    #[inline(always)]
    pub fn get(&self, index: usize) -> i32 {
        // Masks look like this. Ones are the "slots" in the array
        // 111111111111111100000000000000000000000000000000...
        // 000000000000000011111111111111110000000000000000...
        // 000000000000000000000000000000001111111111111111...

        // [0, 3, 0] would look like this:
        // 0000000000000000|0000000000000011|0000000000000000
        let tmp = match index {
            7 => self.v & 0xFFFF0000000000000000000000000000,
            6 => self.v & 0xFFFF000000000000000000000000,
            5 => self.v & 0xFFFF00000000000000000000,
            4 => self.v & 0xFFFF0000000000000000,
            3 => self.v & 0xFFFF000000000000,
            2 => self.v & 0xFFFF00000000,
            1 => self.v & 0xFFFF0000,
            0 => self.v & 0xFFFF,
            _ => panic!("idx > 7"),
        };
        let x = (tmp >> index * 16) as i32;
        if x == 65535 {
            return -1;
        } else {
            x
        }
    }
    #[inline(always)]
    pub fn clear_idx(&mut self, idx: usize) {
        // Clear the bits in the "slot"
        self.v = match idx {
            7 => self.v & !0xFFFF0000000000000000000000000000,
            6 => self.v & !0xFFFF000000000000000000000000,
            5 => self.v & !0xFFFF00000000000000000000,
            4 => self.v & !0xFFFF0000000000000000,
            3 => self.v & !0xFFFF000000000000,
            2 => self.v & !0xFFFF00000000,
            1 => self.v & !0xFFFF0000,
            0 => self.v & !0xFFFF,
            _ => panic!("idx > 7"),
        };
    }
    #[inline(always)]
    pub fn set(&mut self, idx: usize, val: i32) {
        self.clear_idx(idx);
        let mut big = val as u128;
        big <<= idx * 16;
        self.v |= big;
    }
    #[inline(always)]
    pub fn incr_idx(&mut self, idx: usize, incr_amount: i32) {
        self.set(idx, self.get(idx) + incr_amount);
    }
    pub fn new() -> Self {
        U128Arr { v: 0xFFFF }
    }
}

#[cfg(test)]
mod tests {
    use crate::parsing::u128arr::U128Arr;

    #[test]
    fn set1() {
        let mut uv = U128Arr::new();
        let idx = 1;
        uv.set(idx, 5);
        assert_eq!(uv.get(idx), 5);
    }
    #[test]
    fn set2() {
        let mut uv = U128Arr::new();
        let idx = 2;
        uv.set(idx, 5);
        assert_eq!(uv.get(idx), 5);
    }
    #[test]
    fn set3() {
        let mut uv = U128Arr::new();
        let idx = 3;
        uv.set(idx, 5);
        assert_eq!(uv.get(idx), 5);
    }
    #[test]
    fn set4() {
        let mut uv = U128Arr::new();
        let idx = 4;
        uv.set(idx, 5);
        assert_eq!(uv.get(idx), 5);
    }
    #[test]
    fn set5() {
        let mut uv = U128Arr::new();
        let idx = 5;
        uv.set(idx, 5);
        assert_eq!(uv.get(idx), 5);
    }
    #[test]
    fn set6() {
        let mut uv = U128Arr::new();
        let idx = 6;
        uv.set(idx, 5);
        assert_eq!(uv.get(idx), 5);
    }
    #[test]
    fn set7() {
        let mut uv = U128Arr::new();
        let idx = 7;
        uv.set(idx, 5);
        assert_eq!(uv.get(idx), 5);
    }
}
