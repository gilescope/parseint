use super::{
    parse_16_chars, parse_2_chars, parse_32_chars, parse_4_chars, parse_8_chars, trees::*,
    IntErrorKind3, ParseIntError3, MINUS, PLUS,
};

type PIE = ParseIntError3;

#[doc(hidden)]
pub(crate) trait FromStrRadixHelper: PartialOrd + Copy + 'static {
    const MIN: Self;
    const BITS: u32;
    const FIRST_SIG: u8;
    const TAIL: Self;
    const TREE: &'static [Self];
    const CHARS: usize;
    fn from_u128(u: u128) -> Self;
    fn from_u64(u: u64) -> Self;
    fn from_u32(u: u32) -> Self;
    fn from_u16(u: u16) -> Self;
    fn from_u8(u: u8) -> Self;

    fn cchecked_mul(&self, other: Self) -> Option<Self>;
    fn cchecked_sub(&self, other: Self) -> Option<Self>;
    fn cchecked_add(&self, other: Self) -> Option<Self>;
    unsafe fn uunchecked_mul(&self, other: Self) -> Self;
    unsafe fn uunchecked_sub(&self, other: Self) -> Self;
    unsafe fn uunchecked_add(&self, other: Self) -> Self;
}

macro_rules! doit {
    ($($t:ty,$tr:expr,$chars:literal,$first_sig:literal,$tail:literal)*) => ($(impl FromStrRadixHelper for $t {
        const MIN: Self = Self::MIN;
        const FIRST_SIG: u8 = $first_sig;
        const TAIL: Self = $tail;
        const BITS: u32 = Self::BITS;
        const TREE: &'static[Self] = $tr;
        const CHARS: usize = $chars;
        #[inline(always)]
        fn from_u128(u: u128) -> Self { u as Self }
        #[inline(always)]
        fn from_u64(u: u64) -> Self { u as Self }
        #[inline(always)]
        fn from_u32(u: u32) -> Self { u as Self }
        #[inline(always)]
        fn from_u16(u: u16) -> Self { u as Self }
        #[inline(always)]
        fn from_u8(u: u8) -> Self { u as Self }
        #[inline(always)]
        fn cchecked_mul(&self, other: Self) -> Option<Self> {
            Self::checked_mul(*self, other as Self)
        }
        #[inline(always)]
        fn cchecked_sub(&self, other: Self) -> Option<Self> {
            Self::checked_sub(*self, other as Self)
        }
        #[inline(always)]
        fn cchecked_add(&self, other: Self) -> Option<Self> {
            Self::checked_add(*self, other as Self)
        }
        #[inline(always)]
        unsafe fn uunchecked_mul(&self, other: Self) -> Self {
            Self::unchecked_mul(*self, other as Self)
        }
        #[inline(always)]
        unsafe fn uunchecked_sub(&self, other: Self) -> Self {
            Self::unchecked_sub(*self, other as Self)
        }
        #[inline(always)]
        unsafe fn uunchecked_add(&self, other: Self) -> Self {
            Self::unchecked_add(*self, other as Self)
        }
    })*)
}

doit! {
    i8,TENS_I8,3,1,-28
    i16,TENS_I16,5,3,-2768
    i32,TENS_I32,10,2,-147_483_648
    i64,TENS_I64,20,9,-223_372_036_854_775_808
    i128,TENS_I128,39,1,-70141183460469231731687303715884105728
    isize,TENS_ISIZE,20,9,-223_372_036_854_775_808
    u8,TENS_U8,3,2,55
    u16,TENS_U16,5,6,5535
    u32,TENS_U32,10,4,294_967_295
    u64,TENS_U64,20,1,8_446_744_073_709_551_615
    u128,TENS_U128,39,3,40_282_366_920_938_463_463_374_607_431_768_211_455
    usize,TENS_USIZE,20,1,8_446_744_073_709_551_615
}
//TODO: 32bit isize

/// u128: 0 to 340_282_366_920_938_463_463_374_607_431_768_211_455
/// (39 digits!)

pub(crate) fn parse_challenger<T>(mut s: &[u8]) -> Result<T, PIE>
where
    T: FromStrRadixHelper,
{
    let is_signed_ty = T::from_u32(0) > T::MIN;
    unsafe {
        let mut checked: Option<u8> = None;
        if let Some(val) = s.get(0) {
            let mut val = *val;
            let mut res = T::from_u8(0);
            if !(is_signed_ty && val == b'-') {
                loop {
                    let l = s.len();
                    if std::intrinsics::likely(val != b'+' && l < T::CHARS) {
                        let l_1 = l & 1_ != 0 && T::BITS >= 4__;
                        let l_2 = l & 2_ != 0 && T::BITS >= 8__;
                        let l_4 = l & 4_ != 0 && T::BITS >= 16_;
                        let l_8 = l & 8_ != 0 && T::BITS >= 32_;
                        let l16 = l & 16 != 0 && T::BITS >= 64_;
                        let l32 = l & 32 != 0 && T::BITS >= 128;

                        if l_1 {
                            let val = val.wrapping_sub(b'0');
                            let val_t = T::from_u8(val);
                            if std::intrinsics::likely(val <= 9 && l == 1) {
                                return Ok(val_t);
                            } else if std::intrinsics::likely(val <= 9) {
                                s = &s.get_unchecked(1..);
                                res = val_t.uunchecked_mul(*T::TREE.get_unchecked(s.len()));
                            } else {
                                return Err(PIE {
                                    kind: IntErrorKind3::InvalidDigit,
                                });
                            };
                        }
                        if l32 {
                            let val = T::from_u128(parse_32_chars(&s).map_err(|_| PIE {
                                kind: IntErrorKind3::InvalidDigit,
                            })?);
                            s = &s.get_unchecked(32..);
                            res = res
                                .uunchecked_add(T::TREE.get_unchecked(s.len()).uunchecked_mul(val));
                        }
                        if l16 {
                            let val = T::from_u64(parse_16_chars(&s).map_err(|_| PIE {
                                kind: IntErrorKind3::InvalidDigit,
                            })?);
                            s = &s.get_unchecked(16..);
                            res = res
                                .uunchecked_add(T::TREE.get_unchecked(s.len()).uunchecked_mul(val));
                        }
                        if l_8 {
                            let val = T::from_u32(parse_8_chars(&s).map_err(|_| PIE {
                                kind: IntErrorKind3::InvalidDigit,
                            })?);
                            s = &s.get_unchecked(8..);
                            res = res
                                .uunchecked_add(T::TREE.get_unchecked(s.len()).uunchecked_mul(val));
                        }
                        if l_4 {
                            let val = T::from_u16(parse_4_chars(&s).map_err(|_| PIE {
                                kind: IntErrorKind3::InvalidDigit,
                            })?);
                            s = &s.get_unchecked(4..);
                            res = res
                                .uunchecked_add(T::TREE.get_unchecked(s.len()).uunchecked_mul(val));
                        }
                        if l_2 {
                            let val = T::from_u16(parse_2_chars(&s).map_err(|_| PIE {
                                kind: IntErrorKind3::InvalidDigit,
                            })?);
                            //s = &s.get_unchecked(2..);
                            res = res.uunchecked_add(val); //T::TREE.get_unchecked(s.len()).uunchecked_mul
                        }
                        return if checked.is_none() {
                            Ok(res)
                        } else {
                            let checked = T::from_u8(checked.unwrap())
                                .cchecked_mul(*T::TREE.get_unchecked(T::CHARS - 1))
                                .ok_or_else(|| PIE {
                                    kind: IntErrorKind3::InvalidDigit,
                                })?; //PosOverflow})?;
                            checked.cchecked_add(res).ok_or_else(|| PIE {
                                kind: IntErrorKind3::InvalidDigit,
                            }) //PosOverflow})
                        };
                    }

                    if val != b'+' {
                        if val != b'0' {
                            if l == T::CHARS {
                                val = val.wrapping_sub(b'0');
                                if val <= T::FIRST_SIG {
                                    checked = Some(val);
                                    s = &s[1..];
                                    val = *s.get_unchecked(0);
                                    continue;
                                    //return val.cchecked_add(rest).ok_or_else(|| PIE{kind: IntErrorKind3::InvalidDigit })//PosOverflow})
                                } else {
                                    return Err(PIE {
                                        kind: IntErrorKind3::InvalidDigit,
                                    }); //InvalidDigit });
                                }
                            } else {
                                return Err(PIE {
                                    kind: IntErrorKind3::InvalidDigit,
                                }); //PosOverflow });
                            }
                        } else {
                            while val == b'0' {
                                s = &s[1..];
                                val = match s.get(0) {
                                    Some(val) => *val,
                                    None => return Ok(T::from_u8(0)),
                                }
                            }
                            if val == b'+' {
                                return Err(PIE {
                                    kind: IntErrorKind3::InvalidDigit,
                                }); // Empty });
                            }
                        }
                    } else {
                        if checked.is_some() {
                            return Err(PIE {
                                kind: IntErrorKind3::InvalidDigit,
                            });
                        }
                        s = &s[1..];
                        val = match s.get(0) {
                            Some(val) => {
                                if *val == b'+' {
                                    return Err(PIE {
                                        kind: IntErrorKind3::InvalidDigit,
                                    }); // Empty });
                                } else {
                                    *val
                                }
                            }
                            None => {
                                return Err(PIE {
                                    kind: IntErrorKind3::InvalidDigit,
                                })
                            } // Empty });
                        }
                    }
                }
            } else {
                s = &s[1..];
                loop {
                    let l = s.len();

                    if std::intrinsics::likely(l < T::CHARS && l != 0) {
                        if (l & 1) != 0 && T::BITS >= 4__ {
                            let val = s.get_unchecked(0).wrapping_sub(b'0');
                            let val_t = T::from_u8(0).uunchecked_sub(T::from_u8(val));
                            if std::intrinsics::likely(val <= 9 && l == 1) {
                                return Ok(val_t);
                            } else if std::intrinsics::likely(val <= 9) {
                                s = &s.get_unchecked(1..);
                                res = val_t.uunchecked_mul(*T::TREE.get_unchecked(s.len()));
                            } else {
                                return Err(PIE {
                                    kind: IntErrorKind3::InvalidDigit,
                                });
                            };
                        }
                        if (l & 32) != 0 && T::BITS >= 128 {
                            let val = T::from_u128(parse_32_chars(&s).map_err(|_| PIE {
                                kind: IntErrorKind3::InvalidDigit,
                            })?);
                            s = &s.get_unchecked(32..);
                            res = res
                                .uunchecked_sub(T::TREE.get_unchecked(s.len()).uunchecked_mul(val));
                        }
                        if (l & 16) != 0 && T::BITS >= 64_ {
                            let val = T::from_u64(parse_16_chars(&s).map_err(|_| PIE {
                                kind: IntErrorKind3::InvalidDigit,
                            })?);
                            s = &s.get_unchecked(16..);
                            res = res
                                .uunchecked_sub(T::TREE.get_unchecked(s.len()).uunchecked_mul(val));
                        }
                        if (l & 8_) != 0 && T::BITS >= 32_ {
                            let val = T::from_u32(parse_8_chars(&s).map_err(|_| PIE {
                                kind: IntErrorKind3::InvalidDigit,
                            })?);
                            s = &s.get_unchecked(8..);
                            res = res
                                .uunchecked_sub(T::TREE.get_unchecked(s.len()).uunchecked_mul(val));
                        }
                        if (l & 4_) != 0 && T::BITS >= 16_ {
                            let val = T::from_u16(parse_4_chars(&s).map_err(|_| PIE {
                                kind: IntErrorKind3::InvalidDigit,
                            })?);
                            s = &s.get_unchecked(4..);
                            res = res
                                .uunchecked_sub(T::TREE.get_unchecked(s.len()).uunchecked_mul(val));
                        }
                        if (l & 2_ != 0) && T::BITS >= 8__ {
                            let val = T::from_u16(parse_2_chars(&s).map_err(|_| PIE {
                                kind: IntErrorKind3::InvalidDigit,
                            })?);
                            //s = &s.get_unchecked(2..);
                            res = res.uunchecked_sub(val); //T::TREE.get_unchecked(s.len()).uunchecked_mul
                        }
                        return if checked.is_none() {
                            Ok(res)
                        } else {
                            let chk = checked.unwrap();
                            if chk == T::FIRST_SIG && res == T::TAIL {
                                return Ok(T::MIN);
                            }
                            let val = T::from_u8(chk)
                                .cchecked_mul(*T::TREE.get_unchecked(T::CHARS - 1))
                                .ok_or_else(|| PIE {
                                    kind: IntErrorKind3::InvalidDigit,
                                })?; //PosOverflow})?;
                            res.cchecked_sub(val).ok_or_else(|| PIE {
                                kind: IntErrorKind3::InvalidDigit,
                            }) //PosOverflow})
                        };
                    }
                    val = if let Some(val) = s.get(0) {
                        *val
                    } else {
                        return Err(PIE {
                            kind: IntErrorKind3::InvalidDigit,
                        }); //Empty });
                    };
                    if val != b'0' {
                        if l == T::CHARS {
                            val = val.wrapping_sub(b'0');
                            if val <= T::FIRST_SIG {
                                checked = Some(val);
                                s = &s[1..];
                                val = *s.get_unchecked(0);
                                continue;
                            } else {
                                return Err(PIE {
                                    kind: IntErrorKind3::InvalidDigit,
                                }); //InvalidDigit });
                            }
                        } else {
                            return Err(PIE {
                                kind: IntErrorKind3::InvalidDigit,
                            }); //PosOverflow });
                        }
                    } else {
                        while val == b'0' {
                            s = &s[1..];
                            val = match s.get(0) {
                                Some(val) => *val,
                                None => return Ok(T::from_u8(0)),
                            }
                        }
                        if val == b'+' {
                            return Err(PIE {
                                kind: IntErrorKind3::InvalidDigit,
                            }); // Empty });
                        }
                    }
                }
            }
        } else {
            return Err(PIE {
                kind: IntErrorKind3::InvalidDigit,
            }); // Empty });
        }
    }
}

// u128: 0 to 340_282_366_920_938_463_463_374_607_431_768_211_455
// (39 digits!)

macro_rules! invalid {
    () => {
        PIE {
            kind: IntErrorKind3::InvalidDigit,
        }
    };
}
macro_rules! empty {
    () => {
        PIE {
            kind: IntErrorKind3::InvalidDigit,
        }
    };
}

macro_rules! pos_overflow {
    () => {
        PIE {
            kind: IntErrorKind3::InvalidDigit,
        }
    };
}

macro_rules! neg_overflow {
    () => {
        PIE {
            kind: IntErrorKind3::InvalidDigit,
        }
    };
}

pub(crate) fn parse<T>(mut s: &[u8]) -> Result<T, PIE>
where
    T: FromStrRadixHelper,
{
    let is_signed_ty = T::from_u32(0) > T::MIN;
    let mut checked: Option<u8> = None;
    unsafe {
        if let Some(val) = s.get(0) {
            let mut val = val.wrapping_sub(b'0');
            loop {
                if std::intrinsics::likely(val <= 9) {
                    // positive without +. could be long with lots of leading zeros.
                    loop {
                        let l = s.len();
                        if std::intrinsics::likely(l < T::CHARS) {
                            let mut res = T::from_u8(0);
                            let l_1 = l & 1_ != 0 && T::BITS >= 4__;
                            let l_2 = l & 2_ != 0 && T::BITS >= 8__;
                            let l_4 = l & 4_ != 0 && T::BITS >= 16_;
                            let l_8 = l & 8_ != 0 && T::BITS >= 32_;
                            let l16 = l & 16 != 0 && T::BITS >= 64_;
                            let l32 = l & 32 != 0 && T::BITS >= 128;

                            if l_1 {
                                let val_t = T::from_u8(val);
                                s = &s.get_unchecked(1..);
                                if s.is_empty() {
                                    return Ok(val_t);
                                }
                                res = val_t.uunchecked_mul(*T::TREE.get_unchecked(s.len()));
                            }
                            if l_2 {
                                let val = T::from_u16(parse_2_chars(&s).map_err(|_| invalid!())?);
                                s = &s.get_unchecked(2..);
                                if s.is_empty() {
                                    res = res.uunchecked_add(val);
                                    if checked.is_none() {
                                        return Ok(res);
                                    }
                                } else {
                                    res = res.uunchecked_add(T::TREE.get_unchecked(s.len()).uunchecked_mul(val));
                                }
                            }
                            if l_4 {
                                let val = T::from_u16(parse_4_chars(&s).map_err(|_| invalid!())?);
                                s = &s.get_unchecked(4..);
                                if s.is_empty() {
                                    res = res.uunchecked_add(val);
                                    if checked.is_none() {
                                        return Ok(res);
                                    }
                                } else {
                                    res = res.uunchecked_add(
                                        T::TREE.get_unchecked(s.len()).uunchecked_mul(val)
                                    );
                                }
                            }
                            if l_8 {
                                let val = T::from_u32(parse_8_chars(&s).map_err(|_| invalid!())?);
                                s = &s.get_unchecked(8..);
                                if s.is_empty() {
                                    res = res.uunchecked_add(val);
                                    if checked.is_none() {
                                        return Ok(res);
                                    }
                                } else {
                                   res = res.uunchecked_add(
                                        T::TREE.get_unchecked(s.len()).uunchecked_mul(val),
                                    );
                                }
                            }
                            if l16 {
                                let val = T::from_u64(parse_16_chars(&s).map_err(|_| invalid!())?);
                                s = &s.get_unchecked(16..);
                                if s.is_empty() {
                                    res = res.uunchecked_add(val);
                                    if checked.is_none() {
                                        return Ok(res);
                                    }
                                } else {
                                    res = res.uunchecked_add(
                                        T::TREE.get_unchecked(s.len()).uunchecked_mul(val),
                                    );
                                }
                            }
                            if l32 {
                                let val = T::from_u128(parse_32_chars(&s).map_err(|_| invalid!())?);
                                res = res.uunchecked_add(val);
                            }
                            return if checked.is_none() {
                                Ok(res)
                            } else {
                                // SAFETY: mul is in range as `checked` is constrained to <= T::FIRST_SIG
                                let checked = T::from_u8(checked.unwrap())
                                    .uunchecked_mul(*T::TREE.get_unchecked(T::CHARS - 1));
                                checked.cchecked_add(res).ok_or_else(|| pos_overflow!())
                            };
                        }
                        // Deal with edge cases then get back to the top,
                        if l == T::CHARS && val <= T::FIRST_SIG {
                            checked = Some(val);
                            s = &s[1..];
                            val = s.get_unchecked(0).wrapping_sub(b'0');
                            if val > 9 {
                                return Err(invalid!());
                            }
                        } else if val == 0 {
                             // Remove leading zeros
                             val = b'0';
                             while val == b'0' {
                                 s = &s[1..];
                                 val = match s.get(0) {
                                     Some(val) => *val,
                                     None => return Ok(T::from_u8(0)),
                                 }
                             }
                             val = val.wrapping_sub(b'0');
                             if val > 9 {
                                 return Err(empty!());
                             }
                        } else {
                            return Err(pos_overflow!());
                        }

                        debug_assert!(val <= 9);
                    }
                } else if std::intrinsics::likely(is_signed_ty && val == MINUS) {
                    s = &s[1..];

                    // negative without -. could be long with lots of leading zeros.
                    loop {
                        let l = s.len();
                        if std::intrinsics::likely(l < T::CHARS && l != 0) {
                            let mut res = T::from_u8(0);
                            if (l & 1) != 0 && T::BITS >= 4__ {
                                let val = s.get_unchecked(0).wrapping_sub(b'0');
                                let val_t = T::from_u8(0).uunchecked_sub(T::from_u8(val));
                                if std::intrinsics::likely(val <= 9 && l == 1) {
                                    return Ok(val_t);
                                } else if std::intrinsics::likely(val <= 9) {
                                    s = &s.get_unchecked(1..);
                                    res = val_t.uunchecked_mul(*T::TREE.get_unchecked(s.len()));
                                } else {
                                    return Err(invalid!());
                                };
                            }
                            if (l & 2_ != 0) && T::BITS >= 8__ {
                                let val = T::from_u16(parse_2_chars(&s).map_err(|_| invalid!())?);
                                s = &s.get_unchecked(2..);
                                if s.is_empty() {
                                    res = res.uunchecked_sub(val);
                                    if checked.is_none() {
                                        return Ok(res);
                                    }
                                } else {
                                    res = res.uunchecked_sub(T::TREE.get_unchecked(s.len()).uunchecked_mul(val));
                                }
                            }
                            if (l & 4_) != 0 && T::BITS >= 16_ {
                                let val = T::from_u16(parse_4_chars(&s).map_err(|_| invalid!())?);
                                s = &s.get_unchecked(4..);
                                if s.is_empty() {
                                    res = res.uunchecked_sub(val);
                                    if checked.is_none() {
                                        return Ok(res);
                                    }
                                } else {
                                    res = res.uunchecked_sub(
                                        T::TREE.get_unchecked(s.len()).uunchecked_mul(val),
                                    );
                                }
                            }
                            if (l & 8_) != 0 && T::BITS >= 32_ {
                                let val = T::from_u32(parse_8_chars(&s).map_err(|_| invalid!())?);
                                s = &s.get_unchecked(8..);
                                if s.is_empty() {
                                    res = res.uunchecked_sub(val);
                                    if checked.is_none() {
                                        return Ok(res);
                                    }
                                } else {
                                    res = res.uunchecked_sub(
                                        T::TREE.get_unchecked(s.len()).uunchecked_mul(val),
                                    );
                                }
                            }
                            if (l & 16) != 0 && T::BITS >= 64_ {
                                let val = T::from_u64(parse_16_chars(&s).map_err(|_| invalid!())?);
                                s = &s.get_unchecked(16..);
                                if s.is_empty() {
                                    res = res.uunchecked_sub(val);
                                    if checked.is_none() {
                                        return Ok(res);
                                    }
                                } else {
                                    res = res.uunchecked_sub(
                                        T::TREE.get_unchecked(s.len()).uunchecked_mul(val),
                                    );
                                }
                            }
                            if (l & 32) != 0 && T::BITS >= 128 {
                                let val = T::from_u128(parse_32_chars(&s).map_err(|_| invalid!())?);
                                //s = &s.get_unchecked(32..);
                                res = res.uunchecked_sub(val);
                            }
                            return if checked.is_none() {
                                Ok(res)
                            } else {
                                let chk = checked.unwrap();
                                if chk == T::FIRST_SIG && res == T::TAIL {
                                    return Ok(T::MIN);
                                }
                                // SAFETY: mul is in range as `checked` is constrained to <= T::FIRST_SIG
                                let val = T::from_u8(chk)
                                    .uunchecked_mul(*T::TREE.get_unchecked(T::CHARS - 1));
                                res.cchecked_sub(val).ok_or_else(|| neg_overflow!())
                            };
                        }
                        val = if let Some(val) = s.get(0) {
                            *val
                        } else {
                            return Err(empty!());
                        };
                        if val != b'0' {
                            if l == T::CHARS {
                                val = val.wrapping_sub(b'0');
                                if val <= T::FIRST_SIG {
                                    checked = Some(val);
                                    s = &s[1..];
                                } else {
                                    return Err(invalid!());
                                }
                            } else {
                                return Err(neg_overflow!());
                            }
                        } else {
                            while val == b'0' {
                                s = &s[1..];
                                val = match s.get(0) {
                                    Some(val) => *val,
                                    None => return Ok(T::from_u8(0)),
                                }
                            }
                        }
                    }
                } else if val == PLUS {
                    s = &s[1..];
                    val = match s.get(0) {
                        Some(value) => {
                            let value = value.wrapping_sub(b'0');
                            if std::intrinsics::likely(value <= 9) {
                                value
                            } else {
                                return Err(empty!());
                            }
                        }
                        None => return Err(empty!()),
                    }
                } else {
                    return Err(invalid!());
                }
            }
        } else {
            return Err(empty!());
        }
    }
}