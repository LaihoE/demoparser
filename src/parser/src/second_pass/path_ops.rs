use crate::first_pass::read_bits::{Bitreader, DemoParserError};

#[inline(always)]
pub fn generate_fp() -> FieldPath {
    FieldPath {
        path: [-1, 0, 0, 0, 0, 0, 0],
        last: 0,
    }
}

#[inline(always)]
pub fn do_op(opcode: u8, bitreader: &mut Bitreader, field_path: &mut FieldPath) -> Result<(), DemoParserError> {
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
        _ => Err(DemoParserError::UnknownPathOP),
    }
}

#[derive(Clone, Copy, Debug)]
pub struct FieldPath {
    pub path: [i32; 7],
    pub last: usize,
}
impl FieldPath {
    pub fn pop_special(&mut self, n: usize) -> Result<(), DemoParserError> {
        for _ in 0..n {
            *self.get_entry_mut(self.last)? = 0;
            self.last -= 1;
        }
        Ok(())
    }
    pub fn get_entry_mut(&mut self, idx: usize) -> Result<&mut i32, DemoParserError> {
        match self.path.get_mut(idx) {
            Some(entry) => Ok(entry),
            None => return Err(DemoParserError::IllegalPathOp),
        }
    }
}

fn plus_one(_bitreader: &mut Bitreader, field_path: &mut FieldPath) -> Result<(), DemoParserError> {
    *field_path.get_entry_mut(field_path.last)? += 1;
    Ok(())
}

fn plus_two(_bitreader: &mut Bitreader, field_path: &mut FieldPath) -> Result<(), DemoParserError> {
    *field_path.get_entry_mut(field_path.last)? += 2;
    Ok(())
}

fn plus_three(_bitreader: &mut Bitreader, field_path: &mut FieldPath) -> Result<(), DemoParserError> {
    *field_path.get_entry_mut(field_path.last)? += 3;
    Ok(())
}

fn plus_four(_bitreader: &mut Bitreader, field_path: &mut FieldPath) -> Result<(), DemoParserError> {
    *field_path.get_entry_mut(field_path.last)? += 4;
    Ok(())
}

fn plus_n(bitreader: &mut Bitreader, field_path: &mut FieldPath) -> Result<(), DemoParserError> {
    *field_path.get_entry_mut(field_path.last)? += bitreader.read_ubit_var_fp()? as i32 + 5;
    Ok(())
}

fn push_one_left_delta_zero_right_zero(_bitreader: &mut Bitreader, field_path: &mut FieldPath) -> Result<(), DemoParserError> {
    field_path.last += 1;
    *field_path.get_entry_mut(field_path.last)? = 0;
    Ok(())
}

fn push_one_left_delta_zero_right_non_zero(bitreader: &mut Bitreader, field_path: &mut FieldPath) -> Result<(), DemoParserError> {
    field_path.last += 1;
    *field_path.get_entry_mut(field_path.last)? += bitreader.read_ubit_var_fp()? as i32;
    Ok(())
}

fn push_one_left_delta_one_right_zero(_bitreader: &mut Bitreader, field_path: &mut FieldPath) -> Result<(), DemoParserError> {
    *field_path.get_entry_mut(field_path.last)? += 1;
    field_path.last += 1;
    *field_path.get_entry_mut(field_path.last)? = 0;
    Ok(())
}

fn push_one_left_delta_one_right_non_zero(bitreader: &mut Bitreader, field_path: &mut FieldPath) -> Result<(), DemoParserError> {
    *field_path.get_entry_mut(field_path.last)? += 1;
    field_path.last += 1;
    *field_path.get_entry_mut(field_path.last)? = bitreader.read_ubit_var_fp()? as i32;
    Ok(())
}

fn push_one_left_delta_n_right_zero(bitreader: &mut Bitreader, field_path: &mut FieldPath) -> Result<(), DemoParserError> {
    *field_path.get_entry_mut(field_path.last)? += bitreader.read_ubit_var_fp()? as i32;
    field_path.last += 1;
    *field_path.get_entry_mut(field_path.last)? = 0;
    Ok(())
}

fn push_one_left_delta_n_right_non_zero(bitreader: &mut Bitreader, field_path: &mut FieldPath) -> Result<(), DemoParserError> {
    *field_path.get_entry_mut(field_path.last)? += bitreader.read_ubit_var_fp()? as i32 + 2;
    field_path.last += 1;
    *field_path.get_entry_mut(field_path.last)? = bitreader.read_ubit_var_fp()? as i32 + 1;
    Ok(())
}

fn push_one_left_delta_n_right_non_zero_pack6_bits(
    bitreader: &mut Bitreader,
    field_path: &mut FieldPath,
) -> Result<(), DemoParserError> {
    *field_path.get_entry_mut(field_path.last)? += (bitreader.read_nbits(3)? + 2) as i32;
    field_path.last += 1;
    *field_path.get_entry_mut(field_path.last)? = (bitreader.read_nbits(3)? + 1) as i32;
    Ok(())
}

fn push_one_left_delta_n_right_non_zero_pack8_bits(
    bitreader: &mut Bitreader,
    field_path: &mut FieldPath,
) -> Result<(), DemoParserError> {
    *field_path.get_entry_mut(field_path.last)? += (bitreader.read_nbits(4)? + 2) as i32;
    field_path.last += 1;
    *field_path.get_entry_mut(field_path.last)? = (bitreader.read_nbits(4)? + 1) as i32;
    Ok(())
}

fn push_two_left_delta_zero(bitreader: &mut Bitreader, field_path: &mut FieldPath) -> Result<(), DemoParserError> {
    field_path.last += 1;
    *field_path.get_entry_mut(field_path.last)? += bitreader.read_ubit_var_fp()? as i32;
    field_path.last += 1;
    *field_path.get_entry_mut(field_path.last)? += bitreader.read_ubit_var_fp()? as i32;
    Ok(())
}

fn push_two_pack5_left_delta_zero(bitreader: &mut Bitreader, field_path: &mut FieldPath) -> Result<(), DemoParserError> {
    field_path.last += 1;
    *field_path.get_entry_mut(field_path.last)? = bitreader.read_nbits(5)? as i32;
    field_path.last += 1;
    *field_path.get_entry_mut(field_path.last)? = bitreader.read_nbits(5)? as i32;
    Ok(())
}

fn push_three_left_delta_zero(bitreader: &mut Bitreader, field_path: &mut FieldPath) -> Result<(), DemoParserError> {
    field_path.last += 1;
    *field_path.get_entry_mut(field_path.last)? += bitreader.read_ubit_var_fp()? as i32;
    field_path.last += 1;
    *field_path.get_entry_mut(field_path.last)? += bitreader.read_ubit_var_fp()? as i32;
    field_path.last += 1;
    *field_path.get_entry_mut(field_path.last)? += bitreader.read_ubit_var_fp()? as i32;
    Ok(())
}

fn push_three_pack5_left_delta_zero(bitreader: &mut Bitreader, field_path: &mut FieldPath) -> Result<(), DemoParserError> {
    field_path.last += 1;
    *field_path.get_entry_mut(field_path.last)? = bitreader.read_nbits(5)? as i32;
    field_path.last += 1;
    *field_path.get_entry_mut(field_path.last)? = bitreader.read_nbits(5)? as i32;
    field_path.last += 1;
    *field_path.get_entry_mut(field_path.last)? = bitreader.read_nbits(5)? as i32;
    Ok(())
}

fn push_two_left_delta_one(bitreader: &mut Bitreader, field_path: &mut FieldPath) -> Result<(), DemoParserError> {
    *field_path.get_entry_mut(field_path.last)? += 1;
    field_path.last += 1;
    *field_path.get_entry_mut(field_path.last)? += bitreader.read_ubit_var_fp()? as i32;
    field_path.last += 1;
    *field_path.get_entry_mut(field_path.last)? += bitreader.read_ubit_var_fp()? as i32;
    Ok(())
}

fn push_two_pack5_left_delta_one(bitreader: &mut Bitreader, field_path: &mut FieldPath) -> Result<(), DemoParserError> {
    *field_path.get_entry_mut(field_path.last)? += 1;
    field_path.last += 1;
    *field_path.get_entry_mut(field_path.last)? += bitreader.read_nbits(5)? as i32;
    field_path.last += 1;
    *field_path.get_entry_mut(field_path.last)? += bitreader.read_nbits(5)? as i32;
    Ok(())
}

fn push_three_left_delta_one(bitreader: &mut Bitreader, field_path: &mut FieldPath) -> Result<(), DemoParserError> {
    *field_path.get_entry_mut(field_path.last)? += 1;
    field_path.last += 1;
    *field_path.get_entry_mut(field_path.last)? += bitreader.read_ubit_var_fp()? as i32;
    field_path.last += 1;
    *field_path.get_entry_mut(field_path.last)? += bitreader.read_ubit_var_fp()? as i32;
    field_path.last += 1;
    *field_path.get_entry_mut(field_path.last)? += bitreader.read_ubit_var_fp()? as i32;
    Ok(())
}

fn push_three_pack5_left_delta_one(bitreader: &mut Bitreader, field_path: &mut FieldPath) -> Result<(), DemoParserError> {
    *field_path.get_entry_mut(field_path.last)? += 1;
    field_path.last += 1;
    *field_path.get_entry_mut(field_path.last)? += bitreader.read_nbits(5)? as i32;
    field_path.last += 1;
    *field_path.get_entry_mut(field_path.last)? += bitreader.read_nbits(5)? as i32;
    field_path.last += 1;
    *field_path.get_entry_mut(field_path.last)? += bitreader.read_nbits(5)? as i32;
    Ok(())
}

fn push_two_left_delta_n(bitreader: &mut Bitreader, field_path: &mut FieldPath) -> Result<(), DemoParserError> {
    *field_path.get_entry_mut(field_path.last)? += (bitreader.read_u_bit_var()? + 2) as i32;
    field_path.last += 1;
    *field_path.get_entry_mut(field_path.last)? += bitreader.read_ubit_var_fp()? as i32;
    field_path.last += 1;
    *field_path.get_entry_mut(field_path.last)? += bitreader.read_ubit_var_fp()? as i32;
    Ok(())
}

fn push_two_pack5_left_delta_n(bitreader: &mut Bitreader, field_path: &mut FieldPath) -> Result<(), DemoParserError> {
    *field_path.get_entry_mut(field_path.last)? += (bitreader.read_u_bit_var()? + 2) as i32;
    field_path.last += 1;
    *field_path.get_entry_mut(field_path.last)? += bitreader.read_nbits(5)? as i32;
    field_path.last += 1;
    *field_path.get_entry_mut(field_path.last)? += bitreader.read_nbits(5)? as i32;
    Ok(())
}

fn push_three_left_delta_n(bitreader: &mut Bitreader, field_path: &mut FieldPath) -> Result<(), DemoParserError> {
    *field_path.get_entry_mut(field_path.last)? += (bitreader.read_u_bit_var()? + 2) as i32;
    field_path.last += 1;
    *field_path.get_entry_mut(field_path.last)? += bitreader.read_ubit_var_fp()? as i32;
    field_path.last += 1;
    *field_path.get_entry_mut(field_path.last)? += bitreader.read_ubit_var_fp()? as i32;
    field_path.last += 1;
    *field_path.get_entry_mut(field_path.last)? += bitreader.read_ubit_var_fp()? as i32;
    Ok(())
}

fn push_three_pack5_left_delta_n(bitreader: &mut Bitreader, field_path: &mut FieldPath) -> Result<(), DemoParserError> {
    *field_path.get_entry_mut(field_path.last)? += (bitreader.read_u_bit_var()? + 2) as i32;
    field_path.last += 1;
    *field_path.get_entry_mut(field_path.last)? += bitreader.read_nbits(5)? as i32;
    field_path.last += 1;
    *field_path.get_entry_mut(field_path.last)? += bitreader.read_nbits(5)? as i32;
    field_path.last += 1;
    *field_path.get_entry_mut(field_path.last)? += bitreader.read_nbits(5)? as i32;
    Ok(())
}

fn push_n(bitreader: &mut Bitreader, field_path: &mut FieldPath) -> Result<(), DemoParserError> {
    let n = bitreader.read_u_bit_var()? as i32;
    *field_path.get_entry_mut(field_path.last)? += bitreader.read_u_bit_var()? as i32;
    for _ in 0..n {
        field_path.last += 1;
        *field_path.get_entry_mut(field_path.last)? += bitreader.read_ubit_var_fp()? as i32;
    }
    Ok(())
}

fn push_n_and_non_topological(bitreader: &mut Bitreader, field_path: &mut FieldPath) -> Result<(), DemoParserError> {
    for i in 0..field_path.last + 1 {
        if bitreader.read_boolean()? {
            *field_path.get_entry_mut(i)? += bitreader.read_varint32()? + 1;
        }
    }
    let count = bitreader.read_u_bit_var()?;
    for _ in 0..count {
        field_path.last += 1;
        *field_path.get_entry_mut(field_path.last)? = bitreader.read_ubit_var_fp()? as i32;
    }
    Ok(())
}

fn pop_one_plus_one(_bitreader: &mut Bitreader, field_path: &mut FieldPath) -> Result<(), DemoParserError> {
    field_path.pop_special(1)?;
    *field_path.get_entry_mut(field_path.last)? += 1;
    Ok(())
}

fn pop_one_plus_n(bitreader: &mut Bitreader, field_path: &mut FieldPath) -> Result<(), DemoParserError> {
    field_path.pop_special(1)?;
    *field_path.get_entry_mut(field_path.last)? += bitreader.read_ubit_var_fp()? as i32 + 1;
    Ok(())
}

fn pop_all_but_one_plus_one(_bitreader: &mut Bitreader, field_path: &mut FieldPath) -> Result<(), DemoParserError> {
    field_path.pop_special(field_path.last)?;
    *field_path.get_entry_mut(0)? += 1;
    Ok(())
}

fn pop_all_but_one_plus_n(bitreader: &mut Bitreader, field_path: &mut FieldPath) -> Result<(), DemoParserError> {
    field_path.pop_special(field_path.last)?;
    *field_path.get_entry_mut(0)? += bitreader.read_ubit_var_fp()? as i32 + 1;
    Ok(())
}

fn pop_all_but_one_plus_n_pack3_bits(bitreader: &mut Bitreader, field_path: &mut FieldPath) -> Result<(), DemoParserError> {
    field_path.pop_special(field_path.last)?;
    *field_path.get_entry_mut(0)? += bitreader.read_nbits(3)? as i32 + 1;
    Ok(())
}

fn pop_all_but_one_plus_n_pack6_bits(bitreader: &mut Bitreader, field_path: &mut FieldPath) -> Result<(), DemoParserError> {
    field_path.pop_special(field_path.last)?;
    *field_path.get_entry_mut(0)? += bitreader.read_nbits(6)? as i32 + 1;
    Ok(())
}

fn pop_n_plus_one(bitreader: &mut Bitreader, field_path: &mut FieldPath) -> Result<(), DemoParserError> {
    field_path.pop_special(bitreader.read_ubit_var_fp()? as usize)?;
    *field_path.get_entry_mut(field_path.last)? += 1;
    Ok(())
}

fn pop_n_plus_n(bitreader: &mut Bitreader, field_path: &mut FieldPath) -> Result<(), DemoParserError> {
    field_path.pop_special(bitreader.read_ubit_var_fp()? as usize)?;
    *field_path.get_entry_mut(field_path.last)? += bitreader.read_varint32()?;
    Ok(())
}

fn pop_n_and_non_topographical(bitreader: &mut Bitreader, field_path: &mut FieldPath) -> Result<(), DemoParserError> {
    field_path.pop_special(bitreader.read_ubit_var_fp()? as usize)?;
    for i in 0..field_path.last + 1 {
        if bitreader.read_boolean()? {
            *field_path.get_entry_mut(i)? += bitreader.read_varint32()?;
        }
    }
    Ok(())
}

fn non_topo_complex(bitreader: &mut Bitreader, field_path: &mut FieldPath) -> Result<(), DemoParserError> {
    for i in 0..field_path.last + 1 {
        if bitreader.read_boolean()? {
            *field_path.get_entry_mut(i)? += bitreader.read_varint32()?;
        }
    }
    Ok(())
}

fn non_topo_penultimate_plus_one(_bitreader: &mut Bitreader, field_path: &mut FieldPath) -> Result<(), DemoParserError> {
    *field_path.get_entry_mut(field_path.last - 1)? += 1;
    Ok(())
}

fn non_topo_complex_pack4_bits(bitreader: &mut Bitreader, field_path: &mut FieldPath) -> Result<(), DemoParserError> {
    for i in 0..field_path.last + 1 {
        if bitreader.read_boolean()? {
            *field_path.get_entry_mut(i)? += bitreader.read_nbits(4)? as i32 - 7;
        }
    }
    Ok(())
}

/*
Huffman tree is this:

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
