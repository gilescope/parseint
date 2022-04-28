#![no_std]
//#![cfg_attr(all(feature = "nightly", feature = "simd"), feature(stdsimd))]
#![cfg_attr(feature = "nightly", feature(core_intrinsics))]
#![allow(clippy::inconsistent_digit_grouping)]
#![warn(unsafe_op_in_unsafe_fn)]
#[macro_use]
mod parse;

//use log::*;
// mod parse_i16;
// mod parse_i32;
// mod parse_i8;
// mod parse_u16;
// mod parse_u32;
// mod parse_u8;
mod trees;

use core::num::IntErrorKind;

pub use parse::{parse, parse_challenger, parse_from_str, FromStrRadixHelper};
// pub use parse_i16::{parse_i16, parse_i16_challenger};
// pub use parse_i32::{parse_i32, parse_i32_challenger};
// pub use parse_i8::{parse_i8, parse_i8_challenger};
// pub use parse_u16::{parse_u16, parse_u16_challenger};
// pub use parse_u32::{parse_u32, parse_u32_challenger};
// pub use parse_u8::{parse_u8, parse_u8_challenger};

const PLUS: u8 = b'+'.wrapping_sub(b'0');
const MINUS: u8 = b'-'.wrapping_sub(b'0');

/// A public version of `std::num::ParseIntError`
#[derive(Debug, Eq, PartialEq)]
pub struct ParseIntErrorPublic {
    pub kind: IntErrorKind,
}

type Pie = ParseIntErrorPublic;

/// Parse the first 32 chars in a u8 slice as a base 10 integer.
/// SAFETY: Do not call with a string length less than that.
/// SAFETY: slice must be 16 byte aligned.
#[doc(hidden)]
#[cfg(not(all(target_feature = "avx", feature = "simd")))]
#[cfg(target_endian = "little")]
#[inline]
pub unsafe fn parse_32_chars(mut s: &[u8]) -> Result<u128, Pie> {
    debug_assert!(s.len() >= 32);
    let val16 = unsafe { parse_16_chars(s)? as u128 };
    s = &s[16..];
    let res = val16 * 1_0000_0000_0000_0000;

    // Do the same thing again as a parse_32_chars fn would need 256bits.
    let val16 = unsafe { parse_16_chars(s)? as u128 };
    Ok(res + val16)
}

/// For now not going to do simd stuff for big-endien...
/// SAFETY: Do not call with a string length less than that.
/// SAFETY: slice must be 16 byte aligned.
#[doc(hidden)]
#[cfg(not(target_endian = "little"))]
#[inline]
pub unsafe fn parse_32_chars(mut s: &[u8]) -> Result<u128, Pie> {
    debug_assert!(s.len() >= 32);
    let val16 = unsafe { parse_16_chars(s)? as u128 };
    s = &s[16..];
    let res = val16 * 1_0000_0000_0000_0000;

    // Do the same thing again as a parse_32_chars fn would need 256bits.
    let val16 = unsafe { parse_16_chars(s)? as u128 };
    Ok(res + val16)
}

// /// Parse the first 32 chars in a u8 slice as a base 10 integer.
// /// SAFETY: Do not call with a string length less than that.
// #[doc(hidden)]
// #[cfg(all(target_feature = "avx", feature = "simd"))]
// #[inline]
// pub unsafe fn parse_32_chars(s: &[u8]) -> Result<u128, Pie> {
//     debug_assert!(s.len() >= 32);

//     use core::arch::x86_64::{
//         _mm256_hadd_epi32, _mm256_lddqu_si256, _mm256_madd_epi16, _mm256_maddubs_epi16,
//     };
//     use core::mem::transmute;
//     use wide::*;
//     let MULT10: i8x32 = i8x32::from([
//         10_i8, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10_i8, 1, 10, 1, 10, 1, 10, 1,
//         10, 1, 10, 1, 10, 1, 10, 1,
//     ]);

//     let MULT100: i16x16 = i16x16::from([
//         100, 1, 100, 1, 100, 1, 100, 1, 100, 1, 100, 1, 100, 1, 100, 1,
//     ]);
//     let MULT10000: i32x8 = i32x8::from([10000, 1, 10000, 1, 10000, 1, 10000, 1]);
//     let ZEROS: i8x32 = i8x32::splat(b'0' as i8);
//     let ZERO_TO_LOWEST: i8x32 = i8x32::splat(-128);
//     let UPPER_BOUND: i8x32 = i8x32::splat(-128 + 10);

//     let chunk: i8x32 = unsafe { transmute(_mm256_lddqu_si256(core::mem::transmute_copy(&s))) };
//     let chunk = chunk - ZEROS; //will wrap
// chunk stores individual digits
//     let chunk_og = chunk;
//     let digits_at_lowest = chunk_og + ZERO_TO_LOWEST;
//     let range_chk1 = i8x32::cmp_lt(digits_at_lowest, UPPER_BOUND);
//     let x: [u128; 2] = transmute(range_chk1);
//     let is_valid = x[0] == 0 && x[1] == 0; //TODO invert?

// chunk stores individual digits
//     let chunk: i16x16 =
//         unsafe { transmute(_mm256_maddubs_epi16(transmute(chunk), transmute(MULT10))) };
// chunk stores pairs of digits
//     let chunk: i32x8 =
//         unsafe { transmute(_mm256_madd_epi16(transmute(chunk), transmute(MULT100))) };
// chunk stores quads of digits
//     let res = chunk * MULT10000;
//     let chunk: i32x8 = unsafe { transmute(_mm256_hadd_epi32(transmute(res), transmute(res))) };
// chunk stores octs of digits
//     let chunk: [i32; 8] = unsafe { transmute(chunk) };

//     if likely!(is_valid) {
//         let upper = chunk[2] as u64 * 10000_0000_u64 + chunk[3] as u64;
//         let lower = chunk[4] as u64 * 10000_0000_u64 + chunk[5] as u64;

//         let result = upper as u128 * 1_0000_0000_0000_0000_u128 + lower as u128;

//         Ok(result as u128)
//     } else {
//         Err(Pie {
//             kind: IntErrorKind::InvalidDigit,
//         })
//     }
// }

/// Parse the first 32 chars in a u8 slice as a base 10 integer.
/// SAFETY: Do not call with a string length less than that.
#[doc(hidden)]
#[cfg(all(target_feature = "avx", feature = "simd"))]
#[inline]
pub fn parse_32_chars(s: &[u8]) -> Result<u128, Pie> {
    debug_assert!(s.len() >= 32);

    use core::arch::x86_64::{
        _mm256_add_epi8, _mm256_cmpgt_epi8, _mm256_hadd_epi32, _mm256_lddqu_si256,
        _mm256_madd_epi16, _mm256_maddubs_epi16, _mm256_mullo_epi32, _mm256_set1_epi8,
        _mm256_set_epi16, _mm256_set_epi32, _mm256_set_epi8, _mm256_sub_epi16, _mm256_testz_si256,
    };

    unsafe {
        let chunk = _mm256_lddqu_si256(core::mem::transmute_copy(&s));
        let zeros = _mm256_set1_epi8(b'0' as i8);
        let chunk = _mm256_sub_epi16(chunk, zeros); //will wrap
                                                    // chunk stores individual digits

        let zero_to_lowest = _mm256_set1_epi8(-128);
        let digits_at_lowest = _mm256_add_epi8(chunk, zero_to_lowest);

        let upper_bound = _mm256_set1_epi8(-128 + 9);

        let is_valid = _mm256_cmpgt_epi8(digits_at_lowest, upper_bound);
        let is_valid = _mm256_testz_si256(is_valid, is_valid) != 0;

        let mult10 = _mm256_set_epi8(
            1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1,
            10, 1, 10, 1, 10, 1, 10,
        );
        let mult100 = _mm256_set_epi16(
            1, 100, 1, 100, 1, 100, 1, 100, 1, 100, 1, 100, 1, 100, 1, 100,
        );
        let mult10000 = _mm256_set_epi32(1, 10000, 1, 10000, 1, 10000, 1, 10000);

        // chunk stores individual digits
        let chunk = _mm256_maddubs_epi16(chunk, mult10);

        // chunk stores pairs of digits
        //error!("chunk pair: {:?}",fmt_chunk(chunk));
        let chunk = _mm256_madd_epi16(chunk, mult100);

        // chunk stores quads of digits
        // The fuzed mul-add instruction doesn't exist so we need to do it in two steps
        let chunk = _mm256_mullo_epi32(chunk, mult10000);
        let chunk = _mm256_hadd_epi32(chunk, chunk);
        //error!("chunk hadd: {:?}", fmt_chunk32(chunk));

        // chunk stores octs of digits
        let chunk: [i32; 8] = core::mem::transmute(chunk);
        if is_valid {
            // If you're going to move these out of the is_valid block
            // then watch for overflows.
            let upper = chunk[2] as u64 * 10000_0000_u64 + chunk[3] as u64;
            let lower = chunk[4] as u64 * 10000_0000_u64 + chunk[5] as u64;

            let result = upper as u128 * 1_0000_0000_0000_0000_u128 + lower as u128;
            Ok(result)
        } else {
            Err(Pie {
                kind: IntErrorKind::InvalidDigit,
            })
        }
    }
}

// fn fmt_chunk(chunk: core::arch::x86_64::__m256i) -> [u8; 32] {
//     unsafe {
//         core::mem::transmute(chunk)
//     }
// }

// fn fmt_chunk16(chunk: core::arch::x86_64::__m256i) -> [u16; 16] {
//     unsafe {
//         core::mem::transmute(chunk)
//     }
// }
// fn fmt_chunk32(chunk: core::arch::x86_64::__m256i) -> [u32; 8] {
//     unsafe {
//         core::mem::transmute(chunk)
//     }
// }

/// Parse the first 16 chars in a u8 slice as a base 10 integer.
/// SAFETY: Do not call with a string length less than that.
// #[cfg(all(target_feature = "sse2", feature = "simd"))]
// #[inline]
// #[doc(hidden)]
// pub unsafe fn parse_16_chars(s: &[u8]) -> Result<u64, Pie> {
//     debug_assert!(s.len() >= 16);

//     use core::arch::x86_64::{
//         _mm_lddqu_si128, _mm_madd_epi16, _mm_maddubs_epi16, _mm_packus_epi32,
//     };
//     use core::mem::transmute;
//     use safe_arch::*;
//     //TODO: waiting on https://github.com/rust-lang/stdsimd/issues/102
//     let chunk: i8x16 = unsafe { transmute(_mm_lddqu_si128(core::mem::transmute_copy(&s))) };
//     let ZEROS: i8x16 = i8x16::splat(b'0' as i8);

//     let chunk = chunk - ZEROS; //will wrap

//     let ZERO_TO_LOWEST: i8x16 = i8x16::splat(-128);
//     let digits_at_lowest = chunk + ZERO_TO_LOWEST;

//     let UPPER_BOUND: i8x16 = i8x16::splat(-128 + 10);
//     let range_chk1 = i8x16::cmp_lt(digits_at_lowest, UPPER_BOUND);
//     let is_valid = range_chk1.all();

//     let MULT10: i8x16 =
//         unsafe { i8x16::from([10_i8, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1]) };

//     let chunk: i8x16 = unsafe { transmute(_mm_maddubs_epi16(transmute(chunk), transmute(MULT10))) };

//     let MULT100: i16x8 = unsafe { i16x8::from([100, 1, 100, 1, 100, 1, 100, 1]) };

//     let chunk: i16x8 = unsafe { transmute(_mm_madd_epi16(transmute(chunk), transmute(MULT100))) };

//     let chunk = unsafe { _mm_packus_epi32(transmute(chunk), transmute(chunk)) };
//     let MULT10000: i16x8 = unsafe { i16x8::from([10000, 1, 10000, 1, 10000, 1, 10000, 1]) };

//     let chunk: i64x2 = unsafe { transmute(_mm_madd_epi16(chunk, transmute(MULT10000))) };
//     let chunk: [u64; 2] = transmute(chunk); //From::<[i64;2]>(chunk)[1].unsigned_abs(); //this could just be a transmute
//     let chunk = chunk[1];
//     let chunk = ((chunk & 0xffffffff) * 1_0000_0000) + (chunk >> 32);
//     if likely!(is_valid) {
//         Ok(chunk)
//     } else {
//         Err(Pie {
//             kind: IntErrorKind::InvalidDigit,
//         })
//     }
// }

/// Parse the first 16 chars in a u8 slice as a base 10 integer.
/// SAFETY: Do not call with a string length less than that.
/// SAFETY: Aligned / unaligned is fine.
#[cfg(all(target_feature = "sse2", feature = "simd"))]
#[inline]
#[doc(hidden)]
pub unsafe fn parse_16_chars(s: &[u8]) -> Result<u64, Pie> {
    debug_assert!(s.len() >= 16);

    use core::arch::x86_64::{
        _mm_add_epi8, _mm_cmplt_epi8, _mm_cvtsi128_si64, _mm_lddqu_si128, _mm_madd_epi16,
        _mm_maddubs_epi16, _mm_packus_epi32, _mm_set1_epi8, _mm_set_epi16, _mm_set_epi8,
        _mm_sub_epi16, _mm_test_all_ones,
    };

    unsafe {
        let chunk = _mm_lddqu_si128(core::mem::transmute_copy(&s));
        let zeros = _mm_set1_epi8(b'0' as i8);

        let chunk = _mm_sub_epi16(chunk, zeros); //will wrap

        let zero_to_lowest = _mm_set1_epi8(-128);

        // 0 => -128, 1 => -127...
        let digits_at_lowest = _mm_add_epi8(chunk, zero_to_lowest);
        let upper_bound = _mm_set1_epi8(-128 + 10);
        let range_chk1 = _mm_cmplt_epi8(digits_at_lowest, upper_bound);
        let range_chk = _mm_test_all_ones(range_chk1);

        let is_valid = range_chk != 0;
        let mult = _mm_set_epi8(1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10);
        let chunk = _mm_maddubs_epi16(chunk, mult);

        let mult = _mm_set_epi16(1, 100, 1, 100, 1, 100, 1, 100);
        let chunk = _mm_madd_epi16(chunk, mult);

        let chunk = _mm_packus_epi32(chunk, chunk);
        let mult = _mm_set_epi16(0, 0, 0, 0, 1, 10000, 1, 10000);
        let chunk = _mm_madd_epi16(chunk, mult);

        let chunk = _mm_cvtsi128_si64(chunk) as u64;
        let chunk = ((chunk & 0xffffffff) * 1_0000_0000) + (chunk >> 32);

        if is_valid {
            Ok(chunk)
        } else {
            return Err(Pie {
                kind: IntErrorKind::InvalidDigit,
            });
        }
    }
}

/// Parse the first 16 chars in a u8 slice as a base 10 integer.
/// (Almost as good as the simd feature...)
/// SAFETY: Do not call with a string length less than that.
/// SAFETY: slice must be 16 byte aligned.
#[cfg(not(all(target_feature = "sse2", feature = "simd")))]
#[cfg(target_endian = "little")]
#[inline]
#[doc(hidden)]
pub unsafe fn parse_16_chars(s: &[u8]) -> Result<u64, Pie> {
    debug_assert!(s.len() >= 16);
    const MASK_HI: u128 = 0xf0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0u128;
    const ASCII_ZEROS: u128 = 0x30303030303030303030303030303030u128;

    let chunk = unsafe {
        let ptr = s.as_ptr();
        debug_assert!(ptr as usize % core::mem::size_of::<u128>() == 0);
        *(ptr as *const u128)
        //    core::ptr::read_unaligned(ptr as *const u128)
        ^ ASCII_ZEROS
    };
    let chunk_og = chunk;

    // 1-byte mask trick (works on 8 pairs of single digits)
    let lower_digits = (chunk & 0x0f000f000f000f000f000f000f000f00) >> 8;
    let upper_digits = (chunk & 0x000f000f000f000f000f000f000f000f) * 10;
    let chunk = lower_digits + upper_digits;

    // 2-byte mask trick (works on 4 pairs of two digits)
    let lower_digits = (chunk & 0x00ff000000ff000000ff000000ff0000) >> 16;
    let upper_digits = (chunk & 0x000000ff000000ff000000ff000000ff) * 100;
    let chunk = lower_digits + upper_digits;

    // 4-byte mask trick (works on 2 pair of four digits)
    let lower_digits = (chunk & 0x0000ffff000000000000ffff00000000) >> 32;
    let upper_digits = (chunk & 0x000000000000ffff000000000000ffff) * 100_00;
    let chunk = lower_digits + upper_digits;

    let chk = chunk_og.wrapping_add(0x76767676767676767676767676767676u128);
    // 8-byte mask trick (works on a pair of eight digits)
    let lower_digits = ((chunk & 0x00000000ffffffff0000000000000000) >> 64) as u64;
    let upper_digits = (chunk as u64) * 100_00_00_00; //& 0x00000000ffffffff
    let chunk = lower_digits + upper_digits;

    if likely!((chunk_og & MASK_HI) | (chk & 0x80808080808080808080808080808080u128) == 0) {
        Ok(chunk) //u64 can guarantee to contain 19 digits.
    } else {
        Err(Pie {
            kind: IntErrorKind::InvalidDigit,
        })
    }
}

/// Parse the first 16 chars in a u8 slice as a base 10 integer.
/// (Almost as good as the simd feature...)
/// SAFETY: Do not call with a string length less than that.
/// SAFETY: slice must be 16 byte aligned.
#[cfg(not(all(target_feature = "sse2", feature = "simd")))]
#[cfg(not(target_endian = "little"))]
#[inline]
#[doc(hidden)]
pub unsafe fn parse_16_chars(s: &[u8]) -> Result<u64, Pie> {
    debug_assert!(s.len() >= 16);
    const MASK_HI: u128 = 0xf0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0u128;
    const ASCII_ZEROS: u128 = 0x30303030303030303030303030303030u128;

    let chunk = unsafe {
        let ptr = s.as_ptr();
        debug_assert!(ptr as usize % core::mem::size_of::<u128>() == 0);
        *(ptr as *const u128)
        // } else {
        //     panic!("swoops16");

        //     core::ptr::read_unaligned(ptr as *const u128)
        //}) 
        ^ ASCII_ZEROS
    };
    let chunk_og = chunk;

    // 1-byte mask trick (works on 8 pairs of single digits)
    let lower_digits = ((chunk & 0x0f000f000f000f000f000f000f000f00) >> 8) * 10;
    let upper_digits = chunk & 0x000f000f000f000f000f000f000f000f;
    let chunk = lower_digits + upper_digits;

    // 2-byte mask trick (works on 4 pairs of two digits)
    let lower_digits = ((chunk & 0x00ff000000ff000000ff000000ff0000) >> 16) * 100;
    let upper_digits = chunk & 0x000000ff000000ff000000ff000000ff;
    let chunk = lower_digits + upper_digits;

    // 4-byte mask trick (works on 2 pair of four digits)
    let lower_digits = ((chunk & 0x0000ffff000000000000ffff00000000) >> 32) * 100_00;
    let upper_digits = chunk & 0x000000000000ffff000000000000ffff;
    let chunk = lower_digits + upper_digits;

    let chk = chunk_og.wrapping_add(0x76767676767676767676767676767676u128);
    // 8-byte mask trick (works on a pair of eight digits)
    let lower_digits = (((chunk & 0x00000000ffffffff0000000000000000) >> 64) as u64) * 100_00_00_00;
    let upper_digits = chunk as u64; //& 0x00000000ffffffff
    let chunk = lower_digits + upper_digits;

    if likely!((chunk_og & MASK_HI) | (chk & 0x80808080808080808080808080808080u128) == 0) {
        Ok(chunk) //u64 can guarantee to contain 19 digits.
    } else {
        Err(Pie {
            kind: IntErrorKind::InvalidDigit,
        })
    }
}

/// Parse the first 8 chars in a u8 slice as a base 10 integer.
/// SAFETY: Do not call with a string length less than that.
/// SAFETY: slice must be 8 byte aligned.
#[cfg(target_endian = "little")]
#[inline]
#[doc(hidden)]
pub unsafe fn parse_8_chars(s: &[u8]) -> Result<u32, Pie> {
    debug_assert!(s.len() >= 8);
    const MASK_HI: u64 = 0xf0f0f0f0f0f0f0f0u64;
    const ASCII_ZEROS: u64 = 0x3030303030303030u64;
    let ptr = s.as_ptr();
    let chunk = unsafe {
        
        debug_assert!(ptr as usize % core::mem::size_of::<u64>() == 0);
        // (if ptr as usize % core::mem::size_of::<u64>() == 0 {
        *(ptr as *const u64)
        // } else {
        //     panic!("swoops8");

        //     core::ptr::read_unaligned(ptr as *const u64)
        // })
         ^ ASCII_ZEROS
    };
    let valid = (chunk & MASK_HI)
        | (chunk.wrapping_add(0x7676767676767676u64) & 0x8080808080808080u64)
        == 0;

    // 1-byte mask trick (works on 4 pairs of single digits)
    let lower_digits = (chunk & 0x0f000f000f000f00) >> 8;
    let upper_digits = (chunk & 0x000f000f000f000f) * 10; //Compiler does *8 + *2
    let chunk = lower_digits + upper_digits;

    // 2-byte mask trick (works on 2 pairs of two digits)
    let lower_digits = (chunk & 0x00ff000000ff0000) >> 16;
    let upper_digits = (chunk & 0x000000ff000000ff) * 100;
    let chunk = lower_digits + upper_digits;

    // 4-byte mask trick (works on a pair of four digits)
    let lower_digits = ((chunk & 0x0000ffff00000000) >> 32) as u32;
    let upper_digits = (chunk as u32) * 10000; //10000 = 8192 + 1024 + 512 + 256+ 16
                                               //8192 + 2048 + 16 - 256  //& 0x0000ffff

    //We do this before the if shaving 300ps.
    let chunk = lower_digits + upper_digits;

    if likely!(valid) {
        Ok(chunk) //u32 can guarantee to contain 9 digits.
    } else {
        Err(Pie {
            kind: IntErrorKind::InvalidDigit,
        })
    }
}

/// Parse the first 8 chars in a u8 slice as a base 10 integer.
/// SAFETY: Do not call with a string length less than that.
/// SAFETY: slice must be 8 byte aligned.
#[cfg(not(target_endian = "little"))]
#[inline]
#[doc(hidden)]
pub unsafe fn parse_8_chars(s: &[u8]) -> Result<u32, Pie> {
    debug_assert!(s.len() >= 8);
    const MASK_HI: u64 = 0xf0f0f0f0f0f0f0f0u64;
    const ASCII_ZEROS: u64 = 0x3030303030303030u64;

    let chunk = unsafe {
        let ptr = s.as_ptr();
        debug_assert!(ptr as usize % core::mem::size_of::<u64>() == 0);
        //        (if ptr as usize % core::mem::size_of::<u64>() == 0 {
        *(ptr as *const u64)
        // } else {
        //     panic!("swoops8");

        //     core::ptr::read_unaligned(ptr as *const u64)
        // })
         ^ ASCII_ZEROS
    };

    let valid = (chunk & MASK_HI)
        | (chunk.wrapping_add(0x7676767676767676u64) & 0x8080808080808080u64)
        == 0;

    // 1-byte mask trick (works on 4 pairs of single digits)
    let lower_digits = ((chunk & 0x0f000f000f000f00) >> 8) * 10; //Compiler does *8 + *2
    let upper_digits = chunk & 0x000f000f000f000f;
    let chunk = lower_digits + upper_digits;

    // 2-byte mask trick (works on 2 pairs of two digits)
    let lower_digits = ((chunk & 0x00ff000000ff0000) >> 16) * 100; //TODO: decompose * 100 to shifts
    let upper_digits = chunk & 0x000000ff000000ff;
    let chunk = lower_digits + upper_digits;

    // 4-byte mask trick (works on a pair of four digits)
    let lower_digits = (((chunk & 0x0000ffff00000000) >> 32) * 10000) as u32;
    let upper_digits = chunk as u32;

    //We do this before the if shaving 300ps.
    let chunk = lower_digits + upper_digits;

    if likely!(valid) {
        Ok(chunk) //u32 can guarantee to contain 9 digits.
    } else {
        Err(Pie {
            kind: IntErrorKind::InvalidDigit,
        })
    }
}

/// Parse the first 4 chars in a u8 slice as a base 10 integer.
/// SAFETY: Do not call with a string length less than that.
/// SAFETY: slice must be 4 byte aligned.
#[cfg(target_endian = "little")]
#[inline]
#[doc(hidden)]
pub unsafe fn parse_4_chars(s: &[u8]) -> Result<u16, Pie> {
    //SAFETY:
    debug_assert!(s.len() >= 4);

    const MASK_HI: u32 = 0xf0f0f0f0u32;
    const ASCII_ZEROS: u32 = 0x30303030u32;
    let ptr = s.as_ptr() as usize;
    let chunk1 = unsafe {
        
        debug_assert!(ptr as usize % core::mem::size_of::<u32>() == 0);
        // (if ptr % size == 0 {
        *(s.as_ptr() as *const u32)
        // } else {
        //     panic!("swoops4");
        //     core::ptr::read_unaligned(ptr as *const u32)
        // }) 
        ^ ASCII_ZEROS
    };
    // 1-byte mask trick (works on 4 pairs of single digits)
    let lower_digits = (chunk1 & 0x0f000f00) >> 8; // => 0x00f000f0

    let sum = chunk1.wrapping_add(0x76767676u32) & 0x80808080u32;

    let chunk = lower_digits + (chunk1 & 0x000f000f) * 10;

    let masked = chunk as u16; // & 0x00ff;
                               //Next line should be:
                               // let cond = (chunk1 & MASK_HI) | sum == 0;
                               // but hit a wasm bug: https://github.com/rust-lang/rust/issues/85580
    let cond = (chunk1 & MASK_HI) == 0 && sum == 0;

    // Multiply by 100 via shifts
    let m1 = masked << 6;
    let m2 = masked << 5;
    let m3 = masked << 2;

    let r = ((chunk & 0x00ff0000) >> 16) as u16;

    // 2-byte mask trick (works on 2 pairs of two digits)
    let chunk = r + m1 + m2 + m3;

    if likely!(cond) {
        Ok(chunk) //u16 can guarantee to hold 4 digits
    } else {
        Err(Pie {
            kind: IntErrorKind::InvalidDigit,
        })
    }
}

/// Big-endien: test with:
/// ```
/// cargo miri test --target mips64-unknown-linux-gnuabi64
/// ```
/// E.g. "1234" is represented as:
///  0x31323334
///
/// For big endien it's in the right order.
/// SAFETY: minimum of string len 4.
/// SAFETY: slice must be 8 byte aligned.
#[cfg(not(target_endian = "little"))]
#[inline]
#[doc(hidden)]
pub unsafe fn parse_4_chars(s: &[u8]) -> Result<u16, Pie> {
    //SAFETY:
    debug_assert!(s.len() >= 4);

    const MASK_HI: u32 = 0xf0f0f0f0u32;
    const ASCII_ZEROS: u32 = 0x30303030u32;

    let chunk1 = unsafe {
        let ptr = s.as_ptr();
        debug_assert!(ptr as usize % core::mem::size_of::<u32>() == 0);
        //(if ptr as usize % core::mem::size_of::<u32>() == 0 {
        *(ptr as *const u32)
        // } else {
        //     panic!("swoops4");
        //     core::ptr::read_unaligned(ptr as *const u32)
        // })
         ^ ASCII_ZEROS
    };

    // 1-byte mask trick (works on 4 pairs of single digits)
    let tens = (chunk1 & 0x0f000f00) >> 8; // => 0x00f000f0

    let sum = chunk1.wrapping_add(0x76767676u32) & 0x80808080u32;

    let units = chunk1 & 0x000f000f;
    let chunk = tens * 10 + (units);

    let masked = chunk; // & 0x00ff;
                        //Next line should be:
                        // let cond = (chunk1 & MASK_HI) | sum == 0;
                        // but hit a wasm bug: https://github.com/rust-lang/rust/issues/85580
    let cond = (chunk1 & MASK_HI) == 0 && sum == 0;

    // Multiply by 100!
    let m1 = (masked & 0x00ff0000) >> 10; //16 - 6
    let m2 = (masked & 0x00ff0000) >> 11; //16 - 5
    let m3 = (masked & 0x00ff0000) >> 14; //16 - 2

    let r = (chunk & 0x000000ff) as u16;

    // 2-byte mask trick (works on 2 pairs of two digits)
    let chunk = r + (m1 + m2 + m3) as u16;

    if likely!(cond) {
        Ok(chunk) //u16 can guarantee to hold 4 digits
    } else {
        Err(Pie {
            kind: IntErrorKind::InvalidDigit,
        })
    }
}

/// Parse the first 2 chars in a u8 slice as a base 10 integer.
/// (Returning u16 rather than u8 as faster.)
/// SAFETY: Do not call with a string length less than that.
/// SAFETY: slice must be 2 byte aligned.
#[cfg(target_endian = "little")]
#[inline]
#[doc(hidden)]
pub unsafe fn parse_2_chars(s: &[u8]) -> Result<u16, Pie> {
    //SAFETY:
    debug_assert!(s.len() >= 2);
    debug_assert_eq!(s.as_ptr() as usize % core::mem::size_of::<u16>(), 0);
    let chunk = unsafe { *(s.as_ptr() as *const u16) ^ 0x3030u16 };
    //Early add
    let ch = chunk.wrapping_add(0x7676u16);
    //Early calc result before use
    let res = ((chunk & 0x000f) << 1) + ((chunk & 0x000f) << 3) + ((chunk & 0x0f00) >> 8);

    if likely!((chunk & 0xf0f0u16) | (ch & 0x8080u16) == 0) {
        Ok(res)
    } else {
        Err(Pie {
            kind: IntErrorKind::InvalidDigit,
        })
    }
}

/// SAFETY: Do not call with a string length less than that.
/// SAFETY: slice must be 2 byte aligned.
#[cfg(not(target_endian = "little"))]
#[inline]
#[doc(hidden)]
pub unsafe fn parse_2_chars(s: &[u8]) -> Result<u16, Pie> {
    debug_assert!(s.len() >= 2);

    let chunk = unsafe {
        let ptr = s.as_ptr();
        debug_assert!(ptr as usize % core::mem::size_of::<u16>() == 0);
        //(if ptr as usize % core::mem::size_of::<u16>() == 0 {
        *(ptr as *const u16)
        // } else {
        //     panic!("swoops2");
        //     core::ptr::read_unaligned(ptr as *const u16)
        // })
         ^ 0x3030u16
    };
    //Early add
    let ch = chunk.wrapping_add(0x7676u16);
    //Early calc result before use
    //Shift >> 8 is consolidated with *10 shifts: *10 = << 3 +  << 1
    let res = (chunk & 0x000f) + ((chunk & 0x0f00) >> 5) + ((chunk & 0x0f00) >> 7);

    if likely!((chunk & 0xf0f0u16) | (ch & 0x8080u16) == 0) {
        Ok(res)
    } else {
        Err(Pie {
            kind: IntErrorKind::InvalidDigit,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::FromStrRadixHelper;
    use heapless::Vec;
    use numtoa::NumToA;
    use paste::paste;

    // #[wasm_bindgen_test]
    // #[test]
    // fn test_uu128_specific() {
    //     let s = "+000123";
    //     let p: Result<u128, ()> = s.parse().map_err(|_| ());
    //     assert_eq!(
    //         p,
    //         super::parse::<u128>(s.as_bytes()).map_err(|_| ()),
    //         "fail to parse: '{}'",
    //         &s
    //     );
    // }

    /// This test is ignored because it can only really call the fn when
    /// the alignment is right and we're not currently setting up the alignment
    /// so the test is not safe.
    #[test]
    #[ignore]
    fn test_bench32() {
        // A way to dbg! in no_std:
        //        env_logger::init();

        let s = "34028236692093846346337460743176821145";
        assert_eq!(
            Ok(34028236692093846346337460743176_u128),
            unsafe { super::parse_32_chars(s.as_bytes()).map_err(|_| ()) },
            "fail to parse: '{}'",
            &s
        );
    }

    #[test]
    fn test_uu12ddd8_specific() {
        //             340282366920938463463374607431768211455
        //let s = "34028236692093846346337460743176821145";
        let s = "01234567890123456789012345678901234567";
        let p: Result<u128, ()> = s.parse().map_err(|_| ());
        //assert_matches!(p, Ok(_));
        assert_eq!(
            p,
            super::parse::<u128>(s.as_bytes()).map_err(|_| ()),
            "fail to parse: '{}'",
            &s
        );
    }

    macro_rules! gen_tests {
        ($target_type:ty, $min:expr, $max:expr, $step: expr, $max_chars: literal,$postfix: literal, $specific: literal) => {
            paste! {
                #[wasm_bindgen_test]
                #[test]
                fn [<test_ $target_type _specific $postfix>]() {
                    let s = $specific;
                    let p: Result<$target_type, ()> = s.parse().map_err(|_| ());
                    assert_eq!(p, [<parse $postfix>]::<$target_type>(s.as_bytes()).map_err(|_| ()), "fail to parse: '{}'", &s);
                }

                #[wasm_bindgen_test]
                #[test]
                fn [<test_invalid_ascii_ $target_type $postfix>]() {
                    let mut vec = Vec::<_, 42>::new();
                    for &ascii in [b':', b'/'].iter() {
                        for i in 1..$max_chars {
                            vec.clear();
                            for _ in 0..i {
                                vec.push(b'1').unwrap();
                            }
                            for j in 1..i {
                                let mut v = vec.clone();
                                v[j] = ascii;
                                let s = unsafe { core::str::from_utf8_unchecked(&v[..]) };
                                assert_eq!(Err(ParseIntErrorPublic{kind:IntErrorKind::InvalidDigit}), [<parse $postfix>]::<$target_type>(s.as_bytes()), "parsing `{}`", s);
                            }
                        }
                    }
                }

                #[wasm_bindgen_test]
                #[test]
                fn [<test_invalid_too_big_ $target_type $postfix>]() {
                    let mut s = [0u8; 42];
                    let ss = ($target_type::MAX as $target_type).numtoa(10, &mut s);
                    let len = ss.len();
                    s[len] = b'1';
                    let s = unsafe { core::str::from_utf8_unchecked(&s[..len + 1]) };
                    assert_eq!(
                        Err(()),
                        [<parse $postfix>]::<$target_type>(s.as_bytes()).map_err(|_|()),
                        " when parsing '{}'",
                        &s
                    );
                }

                #[wasm_bindgen_test]
                #[test]
                fn [<test_empty_ $target_type $postfix>]() {
                    assert_eq!(
                        Err(Pie {
                            kind: IntErrorKind::Empty
                        }),
                        [<parse $postfix>]::<$target_type>("".as_bytes())
                    );
                }

                //#[wasm_bindgen_test] step too small for wasm
                #[cfg_attr(miri, ignore)]
                #[test]
                fn [<test_ $target_type $postfix>]() {
                    let mut s = [0u8; 42];

                    for i in ($min..$max as $target_type).step_by($step) {
                        let s = unsafe { core::str::from_utf8_unchecked(i.numtoa(10, &mut s)) };
                        let p: Result<$target_type, ()> = s.parse().map_err(|_| ());
                        assert_eq!(p, [<parse $postfix>]::<$target_type>(s.as_bytes()).map_err(|_| ()), "fail to parse: '{}'", &s);
                    }
                }

                #[cfg_attr(miri, ignore)]
                #[test]
                fn [<test_random_ $target_type $postfix>]() {
                    use rand::prelude::*;
                    let mut s = [0u8; 42];
                    let mut rng = rand::thread_rng();

                    for _ in 1..100_000 {
                        let i = rng.gen::<$target_type>();
                        let s = unsafe { core::str::from_utf8_unchecked(i.numtoa(10, &mut s)) };
                        let p: Result<$target_type, ()> = s.parse().map_err(|_| ());
                        assert_eq!(p, [<parse $postfix>]::<$target_type>(s.as_bytes()).map_err(|_| ()), "fail to parse: '{}'", &s);
                    }
                }

                //#[wasm_bindgen_test] step too small for wasm
                #[cfg_attr(miri, ignore)]
                #[test]
                fn [<test_ $target_type _plus $postfix>]() {
                    for i in ($min..$max as $target_type).step_by($step) {
                        let mut s = [0u8; 42];
                        s[0] = b'+';
                        let ss = i.numtoa(10, &mut s[1..]);
                        let len = ss.len();
                        let s = unsafe { core::str::from_utf8_unchecked(&s[0..len+1]) };
                        let p: Result<$target_type, ()> = s.parse().map_err(|_| ());
                        assert_eq!(p, [<parse $postfix>]::<$target_type>(s.as_bytes()).map_err(|_| ()), "fail to parse: '{}'", &s);
                    }
                }

                #[wasm_bindgen_test]
                #[test]
                fn [<test_does_not_accept_plus_after_zero_ $target_type _plus $postfix>]() {
                    let mut s = [0u8; 50];
                    s[s.len() - 1] = b'+';
                    let s = unsafe { core::str::from_utf8_unchecked(&s) };
                    let p: Result<$target_type, ()> = s.parse().map_err(|_| ());
                    assert_eq!(p, [<parse $postfix>]::<$target_type>(s.as_bytes()).map_err(|_| ()), "fail to parse: '{:?}'", &s);
                }

                #[wasm_bindgen_test]
                #[test]
                fn [<test_accepts_many_zeros_ $target_type _plus $postfix>]() {
                    //let i = $max;
                    let s = [0u8; 142];
                    let s = unsafe { core::str::from_utf8_unchecked(&s) };
                    let p: Result<$target_type, ()> = s.parse().map_err(|_| ());
                    assert_eq!(p, [<parse $postfix>]::<$target_type>(s.as_bytes()).map_err(|_| ()), "fail to parse: '{}'", &s);
                }

                #[wasm_bindgen_test]
                #[test]
                fn [<test_accepts_many_leading_zeros_ $target_type _plus $postfix>]() {
                    let mut s = [0u8; 142];
                    let ss = ($target_type::MAX as $target_type).numtoa(10, &mut s[100..]);
                    let len = ss.len();

                    let s = unsafe { core::str::from_utf8_unchecked(&s[..100 + len]) };
                    let p: Result<$target_type, ()> = s.parse().map_err(|_| ());
                    assert_eq!(p, [<parse $postfix>]::<$target_type>(s.as_bytes()).map_err(|_| ()), "fail to parse: '{}'", &s);
                }
            }
        }
    }

    gen_tests!(u8, u8::MIN, u8::MAX, 1, 3, "", "1");
    gen_tests!(u8, u8::MIN, u8::MAX, 1, 3, "_challenger", "+200");

    gen_tests!(i8, i8::MIN, i8::MAX, 1, 3, "", "1");
    gen_tests!(i8, i8::MIN, i8::MAX, 1, 3, "_challenger", "-99");

    gen_tests!(u16, u16::MIN, u16::MAX, 1, 5, "", "1");
    gen_tests!(u16, u16::MIN, u16::MAX, 1, 5, "_challenger", "1");

    gen_tests!(i16, i16::MIN, i16::MAX, 1, 5, "", "1");
    gen_tests!(i16, i16::MIN, i16::MAX, 1, 5, "_challenger", "1");

    gen_tests!(u32, u32::MIN, u32::MAX, 10_301, 10, "", "1");
    gen_tests!(
        u32,
        u32::MIN,
        u32::MAX,
        10_301,
        10,
        "_challenger",
        "4294967295"
    );

    gen_tests!(i32, i32::MIN, i32::MAX, 10_301, 10, "", "-2147483648");
    gen_tests!(i32, i32::MIN, i32::MAX, 10_301, 10, "_challenger", "1");

    #[cfg(target_pointer_width = "16")]
    const LARGE_STEP: usize = 12345;

    #[cfg(target_pointer_width = "32")]
    const LARGE_STEP: usize = usize::MAX;

    #[cfg(target_pointer_width = "64")]
    const LARGE_STEP: usize = 100_301_000_000_000;

    gen_tests!(
        u64,
        u64::MIN,
        u64::MAX,
        LARGE_STEP,
        20,
        "",
        "0000000000000000018446744073709551615"
    );
    gen_tests!(
        u64,
        u64::MIN,
        u64::MAX,
        LARGE_STEP,
        20,
        "_challenger",
        "10000009700000000000"
    );

    gen_tests!(
        i64,
        i64::MIN,
        i64::MAX,
        LARGE_STEP,
        19,
        "",
        "-999993949854775808"
    );

    gen_tests!(i64, i64::MIN, i64::MAX, LARGE_STEP, 19, "_challenger", "1");

    gen_tests!(
        u128,
        u64::MIN as u128,
        u64::MAX,
        LARGE_STEP,
        39,
        "",
        "10000009700000000000"
    );

    gen_tests!(
        u128,
        u64::MIN as u128,
        u64::MAX,
        LARGE_STEP,
        39,
        "_challenger",
        "123456789012345678901234567890123456789"
    );

    gen_tests!(
        i128,
        u64::MIN as i128,
        u64::MAX,
        LARGE_STEP,
        39,
        "",
        "1701411834604692317316873037158841057271" // "1:11111111111111"
    );

    gen_tests!(
        i128,
        u64::MIN as i128,
        u64::MAX,
        LARGE_STEP,
        39,
        "_challenger",
        "123456789012345678901234567890123456789"
    );

    use wasm_bindgen_test::wasm_bindgen_test;

    #[wasm_bindgen_test]
    #[test]
    fn test_fuzz1() {
        // needed checked add when checking digits in range.
        check::<u32, 4>([50, 35, 43, 120]);
    }

    #[wasm_bindgen_test]
    #[test]
    fn test_fuzz2() {
        // Too long is defined by std as invalid digit error rather than overflow.
        check::<u32, 11>([48, 48, 54, 54, 49, 54, 56, 57, 54, 49, 51]);
    }

    #[wasm_bindgen_test]
    #[test]
    fn test_fuzz3() {
        //"00661689613" std can deal with any number of leading zeros
        // as it keeps multiplying them by 10...
        check::<u32, 11>([54, 48, 48, 48, 54, 48, 54, 48, 54, 48, 54]);
    }

    #[wasm_bindgen_test]
    #[test]
    fn test_fuzz4() {
        //leading zeros then plus: "0000+6600660"
        check::<u32, 12>([48, 48, 48, 48, 43, 54, 54, 48, 48, 54, 54, 48]);
    }
    #[wasm_bindgen_test]
    #[test]
    fn test_fuzz5() {
        //leading zeros then plus: "0000+6600660"
        check::<u32, 94>([
            43, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48,
            48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48,
            48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48,
            48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48,
            48, 48, 48, 48, 48, 48,
        ]);
    }
    #[wasm_bindgen_test]
    #[test]
    fn test_fuzz6() {
        check::<u64, 21>([
            43, 49, 43, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48,
        ]);
    }

    #[wasm_bindgen_test]
    #[test]
    fn test_fuzz7() {
        check::<u64, 3>([45, 57, 166]);
    }

    fn check<T, const N: usize>(data: [u8; N])
    where
        T: FromStrRadixHelper,
        T: core::str::FromStr,
        T: core::fmt::Debug,
    {
        let mut s = heapless::String::<N>::new();
        for c in &data {
            if (*c as char).len_utf8() != 1 {
                return;
            }
            s.push(*c as char).unwrap();
        }
        //if let Ok(s) = heapless::String<N>::from_utf8(data.to_vec()) {
        let spec: Result<T, _> = s.parse();
        let expected = spec.map_err(|_| ());
        assert_eq!(expected, parse::<T>(&data).map_err(|_| ()));
        // let expected = spec.map_err(|e| e.kind().clone());
        // assert_eq!(expected, parse_u32(&data).map_err(|e| e.kind));
        // } else {
        //     //just make sure doesn't panic:
        //     let _ = parse::<u32>(&data);
        // }
    }
}
