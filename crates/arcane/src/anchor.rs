use core::num::NonZeroU64;
use core::marker::Copy;
use const_random::const_random;
use derive_more::{From, Into, Deref, DerefMut};
use std::sync::{atomic::{AtomicU64, Ordering}};

static THREAD_ID_IV: u64 = const_random!(u64);
static THREAD_ID_NONCE: AtomicU64 = AtomicU64::new(0);

thread_local! {
    static THREAD_ID: ThreadId = ThreadId::new();
}

pub fn thread_id() -> ThreadId {
    THREAD_ID.with(ToOwned::to_owned)
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Into, Deref)]
#[repr(transparent)]
pub struct ThreadId(NonZeroU64);

mod a {
    use std::num::NonZeroU64;
    use const_random::const_random;
    use core::ops::BitXor;

    const ADD_1: u64 = const_random!(u64);
    const ROT_1: u32 = const_random!(u32);
    const XOR_1: u64 = const_random!(u64);
    const ADD_2: u64 = const_random!(u64);
    const ROT_2: u32 = const_random!(u32);
    const XOR_2: u64 = const_random!(u64);
    const ADD_3: u64 = const_random!(u64);
    const ROT_3: u32 = const_random!(u32);
    const XOR_3: u64 = const_random!(u64);
    const ADD_4: u64 = const_random!(u64);
    const ROT_4: u32 = const_random!(u32);
    const XOR_4: u64 = const_random!(u64);

    pub fn cipher(n: u64) -> u64 {
        n.wrapping_add(ADD_1)
            .rotate_left(ROT_1)
            .bitxor(XOR_1)
            .wrapping_add(ADD_2)
            .rotate_left(ROT_2)
            .bitxor(XOR_2)
            .wrapping_add(ADD_3)
            .rotate_left(ROT_3)
            .bitxor(XOR_3)
            .wrapping_add(ADD_4)
            .rotate_left(ROT_4)
            .bitxor(XOR_4)

    }

    pub fn decipher(n: u64) -> u64 {
        n
    }
}

impl ThreadId {
    fn new() -> Self {
        let nonce = THREAD_ID_NONCE.fetch_add(1, Ordering::Relaxed);
        let plain = THREAD_ID_IV ^ nonce;

        if plain == 0 {
            // 1 in 2^64 chance of happening, skip it to avoid null.
            return ThreadId::new()
        }

        let ones = plain.count_ones();
        debug_assert!(ones >= 1 && ones <= 64);

        let mut mixed = plain;

        let mut previous = false;
        for i in 0..6 {
            let bit = 1 == 1 & (ones >> i);
            if bit == previous {
                if bit {
                    mixed = mixed.rotate_left(3);
                } else {
                    mixed = mixed.rotate_left(7);
                }
            } else {
                if bit {
                    mixed = u64::from_be_bytes(mixed.to_be_bytes().map(|b| b.rotate_left(5)));
                } else {
                    mixed = u64::from_be_bytes(mixed.to_be_bytes().map(|b| b.rotate_left(3)));
                }
            }
            previous = bit;

        }

        match ones {
            // from 1 to 64, we want to mix up the locations of each bit in the 64,
            // but we don't want to change the total number of ones/zeroes.
             => {

                let as_64 = mixed as u64;
                let as_u32 = u32::mixed.to_be_bytes()

                for _ in 1..=ones {
                    // maybe break it down into chunks and rotate those?
                }
            }

            // 0 is unreachable because we check for it above
            // 65 and higher are unreachable because it's a 64-bit integer
            _ => { unreachable!() }

        }

        // INVARIANT: `plain` and `mixed` must have the same number of ones.
        debug_assert_eq!(ones, mixed.count_ones());

        NonZeroU64::new(mixed).map(|n| ThreadId(n)).unwrap_or_else(ThreadId::new)
    }
}
