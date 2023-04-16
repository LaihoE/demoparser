use super::read_bits::BitReaderError;
use super::sendtables::Decoder;
use crate::parsing::parser_settings::Parser;
use crate::parsing::read_bits::Bitreader;
use phf_macros::phf_map;
use smallvec::SmallVec;
use std::cmp::Ordering;
use std::cmp::Reverse;
use std::collections::BinaryHeap;

pub fn do_op(
    opcode: u32,
    bitreader: &mut Bitreader,
    field_path: &mut FieldPath,
) -> Result<(), BitReaderError> {
    // taken directly from here: https://github.com/dotabuff/manta/blob/master/field_path.go
    // Not going to act like I know why exactly these ops are selected, I supposed they provide
    // somewhat good compression.
    match opcode {
        0 => plus_one(bitreader, field_path),
        1 => plus_two(bitreader, field_path),
        2 => plus_three(bitreader, field_path),
        3 => plus_four(bitreader, field_path),
        4 => plus_n(bitreader, field_path),
        5 => push_one_left_delta_zero_right_zero(bitreader, field_path),
        6 => push_one_left_delta_zero_right_non_zero(bitreader, field_path),
        7 => push_one_left_delta_one_right_zero(bitreader, field_path),
        8 => push_one_left_delta_one_right_non_zero(bitreader, field_path),
        9 => push_one_left_delta_n_right_zero(bitreader, field_path),
        10 => push_one_left_delta_n_right_non_zero(bitreader, field_path),
        11 => push_one_left_delta_n_right_non_zero_pack6_bits(bitreader, field_path),
        12 => push_one_left_delta_n_right_non_zero_pack8_bits(bitreader, field_path),
        13 => push_two_left_delta_zero(bitreader, field_path),
        14 => push_two_pack5_left_delta_zero(bitreader, field_path),
        15 => push_three_left_delta_zero(bitreader, field_path),
        16 => push_three_pack5_left_delta_zero(bitreader, field_path),
        17 => push_two_left_delta_one(bitreader, field_path),
        18 => push_two_pack5_left_delta_one(bitreader, field_path),
        19 => push_three_left_delta_one(bitreader, field_path),
        20 => push_three_pack5_left_delta_one(bitreader, field_path),
        21 => push_two_left_delta_n(bitreader, field_path),
        22 => push_two_pack5_left_delta_n(bitreader, field_path),
        23 => push_three_left_delta_n(bitreader, field_path),
        24 => push_three_pack5_left_delta_n(bitreader, field_path),
        25 => push_n(bitreader, field_path),
        26 => push_n_and_non_topological(bitreader, field_path),
        27 => pop_one_plus_one(bitreader, field_path),
        28 => pop_one_plus_n(bitreader, field_path),
        29 => pop_all_but_one_plus_one(bitreader, field_path),
        30 => pop_all_but_one_plus_n(bitreader, field_path),
        31 => pop_all_but_one_plus_n_pack3_bits(bitreader, field_path),
        32 => pop_all_but_one_plus_n_pack6_bits(bitreader, field_path),
        33 => pop_n_plus_one(bitreader, field_path),
        34 => pop_n_plus_n(bitreader, field_path),
        35 => pop_n_and_non_topographical(bitreader, field_path),
        36 => non_topo_complex(bitreader, field_path),
        37 => non_topo_penultimate_plus_one(bitreader, field_path),
        38 => non_topo_complex_pack4_bits(bitreader, field_path),
        _ => Err(BitReaderError::UnknownPathOP),
    }
}

#[derive(Eq, Debug)]
pub struct HuffmanNode {
    pub weight: i32,
    pub value: i32,
    pub left: Option<Box<HuffmanNode>>,
    pub right: Option<Box<HuffmanNode>>,
}

impl HuffmanNode {
    #[inline(always)]
    pub fn is_leaf(&self) -> bool {
        self.left.is_none() && self.right.is_none()
    }
}

impl Ord for HuffmanNode {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.weight == other.weight {
            if self.value >= other.value {
                Ordering::Less
            } else {
                Ordering::Greater
            }
        } else {
            self.weight.cmp(&other.weight)
        }
    }
}

impl PartialOrd for HuffmanNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for HuffmanNode {
    fn eq(&self, other: &Self) -> bool {
        self.weight == other.weight
    }
}

#[derive(Clone, Copy, Debug)]
pub struct FieldPath {
    pub path: [i32; 7],
    pub last: usize,
}
impl FieldPath {
    pub fn pop_special(&mut self, n: usize) {
        for _ in 0..n {
            self.path[self.last] = 0;
            self.last -= 1;
        }
    }
}
#[inline]
fn plus_one(_bitreader: &mut Bitreader, field_path: &mut FieldPath) -> Result<(), BitReaderError> {
    field_path.path[field_path.last] += 1;
    Ok(())
}
#[inline]
fn plus_two(_bitreader: &mut Bitreader, field_path: &mut FieldPath) -> Result<(), BitReaderError> {
    field_path.path[field_path.last] += 2;
    Ok(())
}
#[inline]
fn plus_three(
    _bitreader: &mut Bitreader,
    field_path: &mut FieldPath,
) -> Result<(), BitReaderError> {
    field_path.path[field_path.last] += 3;
    Ok(())
}
#[inline]
fn plus_four(_bitreader: &mut Bitreader, field_path: &mut FieldPath) -> Result<(), BitReaderError> {
    field_path.path[field_path.last] += 4;
    Ok(())
}
#[inline]
fn plus_n(bitreader: &mut Bitreader, field_path: &mut FieldPath) -> Result<(), BitReaderError> {
    field_path.path[field_path.last] += bitreader.read_ubit_var_fp()? as i32 + 5;
    Ok(())
}
#[inline]
fn push_one_left_delta_zero_right_zero(
    _bitreader: &mut Bitreader,
    field_path: &mut FieldPath,
) -> Result<(), BitReaderError> {
    field_path.last += 1;
    field_path.path[field_path.last] = 0;
    Ok(())
}
#[inline]
fn push_one_left_delta_zero_right_non_zero(
    bitreader: &mut Bitreader,
    field_path: &mut FieldPath,
) -> Result<(), BitReaderError> {
    field_path.last += 1;
    field_path.path[field_path.last] += bitreader.read_ubit_var_fp()? as i32;
    Ok(())
}
#[inline]
fn push_one_left_delta_one_right_zero(
    _bitreader: &mut Bitreader,
    field_path: &mut FieldPath,
) -> Result<(), BitReaderError> {
    field_path.path[field_path.last] += 1;
    field_path.last += 1;
    field_path.path[field_path.last] = 0;
    Ok(())
}
#[inline]
fn push_one_left_delta_one_right_non_zero(
    bitreader: &mut Bitreader,
    field_path: &mut FieldPath,
) -> Result<(), BitReaderError> {
    field_path.path[field_path.last] += 1;
    field_path.last += 1;
    field_path.path[field_path.last] = bitreader.read_ubit_var_fp()? as i32;
    Ok(())
}
#[inline]
fn push_one_left_delta_n_right_zero(
    bitreader: &mut Bitreader,
    field_path: &mut FieldPath,
) -> Result<(), BitReaderError> {
    field_path.path[field_path.last] += bitreader.read_ubit_var_fp()? as i32;
    field_path.last += 1;
    field_path.path[field_path.last] = 0;
    Ok(())
}
#[inline]
fn push_one_left_delta_n_right_non_zero(
    bitreader: &mut Bitreader,
    field_path: &mut FieldPath,
) -> Result<(), BitReaderError> {
    field_path.path[field_path.last] += bitreader.read_ubit_var_fp()? as i32 + 2;
    field_path.last += 1;
    field_path.path[field_path.last] = bitreader.read_ubit_var_fp()? as i32 + 1;
    Ok(())
}
#[inline]
fn push_one_left_delta_n_right_non_zero_pack6_bits(
    bitreader: &mut Bitreader,
    field_path: &mut FieldPath,
) -> Result<(), BitReaderError> {
    field_path.path[field_path.last] += (bitreader.read_nbits(3)? + 2) as i32;
    field_path.last += 1;
    field_path.path[field_path.last] = (bitreader.read_nbits(3)? + 1) as i32;
    Ok(())
}
#[inline]
fn push_one_left_delta_n_right_non_zero_pack8_bits(
    bitreader: &mut Bitreader,
    field_path: &mut FieldPath,
) -> Result<(), BitReaderError> {
    field_path.path[field_path.last] += (bitreader.read_nbits(4)? + 2) as i32;
    field_path.last += 1;
    field_path.path[field_path.last] = (bitreader.read_nbits(4)? + 1) as i32;
    Ok(())
}
#[inline]
fn push_two_left_delta_zero(
    bitreader: &mut Bitreader,
    field_path: &mut FieldPath,
) -> Result<(), BitReaderError> {
    field_path.last += 1;
    field_path.path[field_path.last] += bitreader.read_ubit_var_fp()? as i32;
    field_path.last += 1;
    field_path.path[field_path.last] += bitreader.read_ubit_var_fp()? as i32;
    Ok(())
}
#[inline]
fn push_two_pack5_left_delta_zero(
    bitreader: &mut Bitreader,
    field_path: &mut FieldPath,
) -> Result<(), BitReaderError> {
    field_path.last += 1;
    field_path.path[field_path.last] = bitreader.read_nbits(5)? as i32;
    field_path.last += 1;
    field_path.path[field_path.last] = bitreader.read_nbits(5)? as i32;
    Ok(())
}
#[inline]
fn push_three_left_delta_zero(
    bitreader: &mut Bitreader,
    field_path: &mut FieldPath,
) -> Result<(), BitReaderError> {
    field_path.last += 1;
    field_path.path[field_path.last] += bitreader.read_ubit_var_fp()? as i32;
    field_path.last += 1;
    field_path.path[field_path.last] += bitreader.read_ubit_var_fp()? as i32;
    field_path.last += 1;
    field_path.path[field_path.last] += bitreader.read_ubit_var_fp()? as i32;
    Ok(())
}
#[inline]
fn push_three_pack5_left_delta_zero(
    bitreader: &mut Bitreader,
    field_path: &mut FieldPath,
) -> Result<(), BitReaderError> {
    field_path.last += 1;
    field_path.path[field_path.last] = bitreader.read_nbits(5)? as i32;
    field_path.last += 1;
    field_path.path[field_path.last] = bitreader.read_nbits(5)? as i32;
    field_path.last += 1;
    field_path.path[field_path.last] = bitreader.read_nbits(5)? as i32;
    Ok(())
}
#[inline]
fn push_two_left_delta_one(
    bitreader: &mut Bitreader,
    field_path: &mut FieldPath,
) -> Result<(), BitReaderError> {
    field_path.path[field_path.last] += 1;
    field_path.last += 1;
    field_path.path[field_path.last] += bitreader.read_ubit_var_fp()? as i32;
    field_path.last += 1;
    field_path.path[field_path.last] += bitreader.read_ubit_var_fp()? as i32;
    Ok(())
}
#[inline]
fn push_two_pack5_left_delta_one(
    bitreader: &mut Bitreader,
    field_path: &mut FieldPath,
) -> Result<(), BitReaderError> {
    field_path.path[field_path.last] += 1;
    field_path.last += 1;
    field_path.path[field_path.last] += bitreader.read_nbits(5)? as i32;
    field_path.last += 1;
    field_path.path[field_path.last] += bitreader.read_nbits(5)? as i32;
    Ok(())
}
#[inline]
fn push_three_left_delta_one(
    bitreader: &mut Bitreader,
    field_path: &mut FieldPath,
) -> Result<(), BitReaderError> {
    field_path.path[field_path.last] += 1;
    field_path.last += 1;
    field_path.path[field_path.last] += bitreader.read_ubit_var_fp()? as i32;
    field_path.last += 1;
    field_path.path[field_path.last] += bitreader.read_ubit_var_fp()? as i32;
    field_path.last += 1;
    field_path.path[field_path.last] += bitreader.read_ubit_var_fp()? as i32;
    Ok(())
}
#[inline]
fn push_three_pack5_left_delta_one(
    bitreader: &mut Bitreader,
    field_path: &mut FieldPath,
) -> Result<(), BitReaderError> {
    field_path.path[field_path.last] += 1;
    field_path.last += 1;
    field_path.path[field_path.last] += bitreader.read_nbits(5)? as i32;
    field_path.last += 1;
    field_path.path[field_path.last] += bitreader.read_nbits(5)? as i32;
    field_path.last += 1;
    field_path.path[field_path.last] += bitreader.read_nbits(5)? as i32;
    Ok(())
}
#[inline]
fn push_two_left_delta_n(
    bitreader: &mut Bitreader,
    field_path: &mut FieldPath,
) -> Result<(), BitReaderError> {
    field_path.path[field_path.last] += (bitreader.read_u_bit_var()? + 2) as i32;
    field_path.last += 1;
    field_path.path[field_path.last] += bitreader.read_ubit_var_fp()? as i32;
    field_path.last += 1;
    field_path.path[field_path.last] += bitreader.read_ubit_var_fp()? as i32;
    Ok(())
}
#[inline]
fn push_two_pack5_left_delta_n(
    bitreader: &mut Bitreader,
    field_path: &mut FieldPath,
) -> Result<(), BitReaderError> {
    field_path.path[field_path.last] += (bitreader.read_u_bit_var()? + 2) as i32;
    field_path.last += 1;
    field_path.path[field_path.last] += bitreader.read_nbits(5)? as i32;
    field_path.last += 1;
    field_path.path[field_path.last] += bitreader.read_nbits(5)? as i32;
    Ok(())
}
#[inline]
fn push_three_left_delta_n(
    bitreader: &mut Bitreader,
    field_path: &mut FieldPath,
) -> Result<(), BitReaderError> {
    field_path.path[field_path.last] += (bitreader.read_u_bit_var()? + 2) as i32;
    field_path.last += 1;
    field_path.path[field_path.last] += bitreader.read_ubit_var_fp()? as i32;
    field_path.last += 1;
    field_path.path[field_path.last] += bitreader.read_ubit_var_fp()? as i32;
    field_path.last += 1;
    field_path.path[field_path.last] += bitreader.read_ubit_var_fp()? as i32;
    Ok(())
}
#[inline]
fn push_three_pack5_left_delta_n(
    bitreader: &mut Bitreader,
    field_path: &mut FieldPath,
) -> Result<(), BitReaderError> {
    field_path.path[field_path.last] += (bitreader.read_u_bit_var()? + 2) as i32;
    field_path.last += 1;
    field_path.path[field_path.last] += bitreader.read_nbits(5)? as i32;
    field_path.last += 1;
    field_path.path[field_path.last] += bitreader.read_nbits(5)? as i32;
    field_path.last += 1;
    field_path.path[field_path.last] += bitreader.read_nbits(5)? as i32;
    Ok(())
}
#[inline]
fn push_n(bitreader: &mut Bitreader, field_path: &mut FieldPath) -> Result<(), BitReaderError> {
    let n = bitreader.read_u_bit_var()? as i32;
    field_path.path[field_path.last] += bitreader.read_u_bit_var()? as i32;
    for _ in 0..n {
        field_path.last += 1;
        field_path.path[field_path.last] += bitreader.read_ubit_var_fp()? as i32;
    }
    Ok(())
}
#[inline]
fn push_n_and_non_topological(
    bitreader: &mut Bitreader,
    field_path: &mut FieldPath,
) -> Result<(), BitReaderError> {
    for i in 0..field_path.last + 1 {
        if bitreader.read_boolie()? {
            field_path.path[i] += bitreader.read_varint32()? + 1;
        }
    }
    let count = bitreader.read_u_bit_var()?;
    for _ in 0..count {
        field_path.last += 1;
        field_path.path[field_path.last] = bitreader.read_ubit_var_fp()? as i32;
    }
    Ok(())
}
#[inline]
fn pop_one_plus_one(
    _bitreader: &mut Bitreader,
    field_path: &mut FieldPath,
) -> Result<(), BitReaderError> {
    field_path.pop_special(1);
    field_path.path[field_path.last] += 1;
    Ok(())
}
#[inline]
fn pop_one_plus_n(
    bitreader: &mut Bitreader,
    field_path: &mut FieldPath,
) -> Result<(), BitReaderError> {
    field_path.pop_special(1);
    field_path.path[field_path.last] += bitreader.read_ubit_var_fp()? as i32 + 1;
    Ok(())
}
#[inline]
fn pop_all_but_one_plus_one(
    _bitreader: &mut Bitreader,
    field_path: &mut FieldPath,
) -> Result<(), BitReaderError> {
    field_path.pop_special(field_path.last);
    field_path.path[0] += 1;
    Ok(())
}
#[inline]
fn pop_all_but_one_plus_n(
    bitreader: &mut Bitreader,
    field_path: &mut FieldPath,
) -> Result<(), BitReaderError> {
    field_path.pop_special(field_path.last);
    field_path.path[0] += bitreader.read_ubit_var_fp()? as i32 + 1;
    Ok(())
}
#[inline]
fn pop_all_but_one_plus_n_pack3_bits(
    bitreader: &mut Bitreader,
    field_path: &mut FieldPath,
) -> Result<(), BitReaderError> {
    field_path.pop_special(field_path.last);
    field_path.path[0] += bitreader.read_nbits(3)? as i32 + 1;
    Ok(())
}
#[inline]
fn pop_all_but_one_plus_n_pack6_bits(
    bitreader: &mut Bitreader,
    field_path: &mut FieldPath,
) -> Result<(), BitReaderError> {
    field_path.pop_special(field_path.last);
    field_path.path[0] += bitreader.read_nbits(6)? as i32 + 1;
    Ok(())
}
#[inline]
fn pop_n_plus_one(
    bitreader: &mut Bitreader,
    field_path: &mut FieldPath,
) -> Result<(), BitReaderError> {
    field_path.pop_special(bitreader.read_ubit_var_fp()? as usize);
    field_path.path[field_path.last] += 1;
    Ok(())
}
#[inline]
fn pop_n_plus_n(
    bitreader: &mut Bitreader,
    field_path: &mut FieldPath,
) -> Result<(), BitReaderError> {
    field_path.pop_special(bitreader.read_ubit_var_fp()? as usize);
    field_path.path[field_path.last] += bitreader.read_varint32()?;
    Ok(())
}
#[inline]
fn pop_n_and_non_topographical(
    bitreader: &mut Bitreader,
    field_path: &mut FieldPath,
) -> Result<(), BitReaderError> {
    field_path.pop_special(bitreader.read_ubit_var_fp()? as usize);
    for i in 0..field_path.last + 1 {
        if bitreader.read_boolie()? {
            field_path.path[i] += bitreader.read_varint32()?;
        }
    }
    Ok(())
}
#[inline]
fn non_topo_complex(
    bitreader: &mut Bitreader,
    field_path: &mut FieldPath,
) -> Result<(), BitReaderError> {
    for i in 0..field_path.last + 1 {
        if bitreader.read_boolie()? {
            field_path.path[i] += bitreader.read_varint32()?;
        }
    }
    Ok(())
}
#[inline]
fn non_topo_penultimate_plus_one(
    _bitreader: &mut Bitreader,
    field_path: &mut FieldPath,
) -> Result<(), BitReaderError> {
    field_path.path[field_path.last - 1] += 1;
    Ok(())
}
#[inline]
fn non_topo_complex_pack4_bits(
    bitreader: &mut Bitreader,
    field_path: &mut FieldPath,
) -> Result<(), BitReaderError> {
    for i in 0..field_path.last + 1 {
        if bitreader.read_boolie()? {
            field_path.path[i] += bitreader.read_nbits(4)? as i32 - 7;
        }
    }
    Ok(())
}

const PAIRS: [(&str, i32); 40] = [
    ("PlusOne", 36271),
    ("PlusTwo", 10334),
    ("PlusThree", 1375),
    ("PlusFour", 646),
    ("PlusN", 4128),
    ("PushOneLeftDeltaZeroRightZero", 35),
    ("PushOneLeftDeltaZeroRightNonZero", 3),
    ("PushOneLeftDeltaOneRightZero", 521),
    ("PushOneLeftDeltaOneRightNonZero", 2942),
    ("PushOneLeftDeltaNRightZero", 560),
    ("PushOneLeftDeltaNRightNonZero", 471),
    ("PushOneLeftDeltaNRightNonZeroPack6Bits", 10530),
    ("PushOneLeftDeltaNRightNonZeroPack8Bits", 251),
    ("PushTwoLeftDeltaZero", 0),
    ("PushTwoPack5LeftDeltaZero", 0),
    ("PushThreeLeftDeltaZero", 0),
    ("PushThreePack5LeftDeltaZero", 0),
    ("PushTwoLeftDeltaOne", 0),
    ("PushTwoPack5LeftDeltaOne", 0),
    ("PushThreeLeftDeltaOne", 0),
    ("PushThreePack5LeftDeltaOne", 0),
    ("PushTwoLeftDeltaN", 0),
    ("PushTwoPack5LeftDeltaN", 0),
    ("PushThreeLeftDeltaN", 0),
    ("PushThreePack5LeftDeltaN", 0),
    ("PushN", 0),
    ("PushNAndNonTopological", 310),
    ("PopOnePlusOne", 2),
    ("PopOnePlusN", 0),
    ("PopAllButOnePlusOne", 1837),
    ("PopAllButOnePlusN", 149),
    ("PopAllButOnePlusNPack3Bits", 300),
    ("PopAllButOnePlusNPack6Bits", 634),
    ("PopNPlusOne", 0),
    ("PopNPlusN", 0),
    ("PopNAndNonTopographical", 1),
    ("NonTopoComplex", 76),
    ("NonTopoPenultimatePlusOne", 271),
    ("NonTopoComplexPack4Bits", 99),
    ("FieldPathEncodeFinish", 25474),
];

pub static _BITPATTERN: phf::Map<i32, i32> = phf_map! {
0i32 => 0 ,
39i32 => -2 ,
8i32 => -8 ,
2i32 => -14 ,
29i32 => -13 ,
4i32 => -6 ,
30i32 => -80 ,
38i32 => -158 ,
35i32 => -10048 ,
34i32 => -10047 ,
27i32 => -5023 ,
25i32 => -10044 ,
24i32 => -10043 ,
33i32 => -10042 ,
28i32 => -10041 ,
13i32 => -10040 ,
15i32 => -20078 ,
14i32 => -20077 ,
6i32 => -5019 ,
21i32 => -20072 ,
20i32 => -20071 ,
23i32 => -20070 ,
22i32 => -20069 ,
17i32 => -20068 ,
16i32 => -20067 ,
19i32 => -20066 ,
18i32 => -20065 ,
5i32 => -627 ,
36i32 => -313 ,
10i32 => -39 ,
7i32 => -38 ,
12i32 => -74 ,
37i32 => -73 ,
9i32 => -36 ,
31i32 => -70 ,
26i32 => -69 ,
32i32 => -34 ,
3i32 => -33 ,
1i32 => -2 ,
11i32 => -1 ,
};

pub fn generate_huffman_tree() -> Option<HuffmanNode> {
    /*
    Should result in this tree (same as Dotabuffs tree):

    value, weight, len(prefix), prefix
    0	36271	2	0
    39	25474	3	10
    8	2942	6	11000
    2	1375	7	110010
    29	1837	7	110011
    4	4128	6	11010
    30	149	    10	110110000
    38	99	    11	1101100010
    35	1	    17	1101100011000000
    34	1	    17	1101100011000001
    27	2	    16	110110001100001
    25	1	    17	1101100011000100
    24	1	    17	1101100011000101
    33	1	    17	1101100011000110
    28	1	    17	1101100011000111
    13	1	    17	1101100011001000
    15	1	    18	11011000110010010
    14	1	    18	11011000110010011
    6	3	    16	110110001100101
    21	1	    18	11011000110011000
    20	1	    18	11011000110011001
    23	1	    18	11011000110011010
    22	1	    18	11011000110011011
    17	1	    18	11011000110011100
    16	1	    18	11011000110011101
    19	1	    18	11011000110011110
    18	1	    18	11011000110011111
    5	35	    13	110110001101
    36	76	    12	11011000111
    10	471	    9	11011001
    7	521	    9	11011010
    12	251	    10	110110110
    37	271	    10	110110111
    9	560	    9	11011100
    31	300	    10	110111010
    26	310	    10	110111011
    32	634	    9	11011110
    3	646	    9	11011111
    1	10334	5	1110
    11	10530	5	1111
    */

    /*
    0,0
    39,10
    8,11000
    2,110010
    29,110011
    4,11010
    30,110110000
    38,1101100010
    35,1101100011000000
    34,1101100011000001
    27,110110001100001
    25,1101100011000100
    24,1101100011000101
    33,1101100011000110
    28,1101100011000111
    13,1101100011001000
    15,11011000110010010
    14,11011000110010011
    6,110110001100101
    21,11011000110011000
    20,11011000110011001
    23,11011000110011010
    22,11011000110011011
    17,11011000110011100
    16,11011000110011101
    19,11011000110011110
    18,11011000110011111
    5,110110001101
    36,11011000111
    10,11011001
    7,11011010
    12,110110110
    37,110110111
    9,11011100
    31,110111010
    26,110111011
    32,11011110
    3,11011111
    1,1110
    11,1111

    */

    let mut trees = vec![];
    for (idx, (_, weight)) in PAIRS.iter().enumerate() {
        let node = if *weight == 0 {
            HuffmanNode {
                weight: 1,
                value: idx as i32,
                left: None,
                right: None,
            }
        } else {
            HuffmanNode {
                weight: *weight,
                value: idx as i32,
                left: None,
                right: None,
            }
        };
        trees.push(node);
    }

    let mut heap = BinaryHeap::new();
    for tree in trees {
        heap.push(Reverse(tree));
    }

    for idx in 0..heap.len() - 1 {
        let a = heap.pop().unwrap();
        let b = heap.pop().unwrap();
        heap.push(Reverse(HuffmanNode {
            weight: a.0.weight + b.0.weight,
            value: (idx + 40) as i32,
            left: Some(Box::new(a.0)),
            right: Some(Box::new(b.0)),
        }))
    }
    Some(heap.pop().unwrap().0)
}

#[allow(dead_code)]
impl Parser {
    pub fn print_tree(&self, tree: Option<&HuffmanNode>, prefix: Vec<i32>) {
        match tree {
            None => return,
            Some(t) => {
                if t.is_leaf() {
                    println!(" ");
                    print!("{} {} ", t.value, t.weight);
                    for i in prefix {
                        print!("{}", i);
                    }
                    //println!(" ")
                } else {
                    let mut l = prefix.clone();
                    let mut r = prefix.clone();
                    l.push(0);
                    r.push(1);
                    self.print_tree(Some(&t.left.as_ref().unwrap()), l);
                    self.print_tree(Some(&t.right.as_ref().unwrap()), r);
                }
            }
        }
    }
}
