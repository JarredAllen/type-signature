//! Implementation of signatures for types.
//!
//! See the [`TypeSignature`] trait for more details.

#![no_std]

/// A type that can be made into a signature.
///
/// If implementing for a custom type, please use the derive macro.
pub trait TypeSignature {
    /// The signature of this type.
    ///
    /// For a fixed type, if the hash of this value as produced by the derive macro changes, it
    /// will be treated as a breaking change.
    const SIGNATURE: TypeSignatureHasher;

    /// A const-available u64 value.
    ///
    /// For a fixed type, if this value as produced by the derive macro changes, it will be treated
    /// as a breaking change.
    const CONST_HASH: u64 = Self::SIGNATURE.const_hash();
}

pub use type_signature_derive::TypeSignature;

/// A hashable type for generating a signature for a type.
#[derive(Hash)]
pub struct TypeSignatureHasher {
    /// The name of the type being hashed.
    #[doc(hidden)]
    pub ty_name: &'static str,
    /// The types of the generic arguments.
    #[doc(hidden)]
    pub ty_generics: &'static [&'static TypeSignatureHasher],
    /// Hashes of const generics.
    #[doc(hidden)]
    pub const_generic_hashes: &'static [u64],
    /// The "variants" of a type.
    ///
    /// For each "variant", it has the name of the variant and a list of the field names and types.
    #[doc(hidden)]
    pub variants: &'static [(
        &'static str,
        &'static [(&'static str, &'static TypeSignatureHasher)],
    )],
}
impl TypeSignatureHasher {
    /// Generate a hash at compile time.
    ///
    /// This function exists to cover for the inability to call [`core::hash::Hash::hash`] at const-time, and
    /// will likely be deprecated once const traits exist.
    #[must_use]
    pub const fn const_hash(&self) -> u64 {
        let mut accumulator = 0x1b6142dc880364ed;

        // Mix in the name of the type
        __macro_export::mix_values(&mut accumulator, __macro_export::hash_str(self.ty_name));

        // Mix in the types of each generic.
        {
            let mut generic_idx = 0;
            while generic_idx < self.ty_generics.len() {
                __macro_export::mix_values(
                    &mut accumulator,
                    self.ty_generics[generic_idx].const_hash(),
                );
                generic_idx += 1;
            }
        }

        // Mix in each const generic argument
        {
            let mut const_generic_idx = 0;
            while const_generic_idx < self.const_generic_hashes.len() {
                __macro_export::mix_values(
                    &mut accumulator,
                    self.const_generic_hashes[const_generic_idx],
                );
                const_generic_idx += 1;
            }
        }

        // Mix in the types and names of each field.
        {
            let mut variant_idx = 0;
            while variant_idx < self.variants.len() {
                let (variant_name, variant_fields) = self.variants[variant_idx];
                __macro_export::mix_values(
                    &mut accumulator,
                    __macro_export::hash_str(variant_name),
                );
                let mut field_idx = 0;
                while field_idx < variant_fields.len() {
                    let (field_name, field_hasher) = variant_fields[field_idx];
                    __macro_export::mix_values(
                        &mut accumulator,
                        __macro_export::hash_str(field_name),
                    );
                    __macro_export::mix_values(&mut accumulator, field_hasher.const_hash());
                    field_idx += 1;
                }
                variant_idx += 1;
            }
        }

        accumulator
    }
}

/// Provide an implementation for a stdlib type.
macro_rules! impl_for_stdlib_ty {
    ($(
        $stdty:ty $(where < $( $generic:ident $( : $generic_cond:tt )? ),* > )?
    ),+ $(,)?) => {$(
        impl$( < $( $generic $( : $generic_cond )? ),* > )? $crate::TypeSignature for $stdty {
            const SIGNATURE: $crate::TypeSignatureHasher  = $crate::TypeSignatureHasher {
                ty_name: stringify!($crate::TypeSignature impl for $stdty),
                ty_generics: &[
                    $( $( &<$generic as $crate::TypeSignature>::SIGNATURE, )* )?
                ],
                const_generic_hashes: &[],
                // Not formally correct, but good enough for stdlib types since they won't change
                variants: &[],
            };
        }
    )+};
}

impl_for_stdlib_ty!(
    u8, u16, u32, usize, u64, u128,
    i8, i16, i32, isize, i64, i128,
    bool,
    f32,
    f64,
    char,
    str,
    (),
    &T where <T: TypeSignature>,
    &mut T where <T: TypeSignature>,
    *const T where <T: TypeSignature>,
    *mut T where <T: TypeSignature>,
    [T] where <T: TypeSignature>,
    Option<T> where <T: TypeSignature> ,
    Result<T, E> where <T: TypeSignature, E: TypeSignature>,
    core::marker::PhantomData<T> where <T: TypeSignature>,
    core::mem::MaybeUninit<T> where <T: TypeSignature>,
    core::mem::ManuallyDrop<T> where <T: TypeSignature>,
    core::net::IpAddr, core::net::Ipv4Addr, core::net::Ipv6Addr,
    core::net::SocketAddr, core::net::SocketAddrV4, core::net::SocketAddrV6,
    core::num::NonZeroU8, core::num::NonZeroU16, core::num::NonZeroU32, core::num::NonZeroU64, core::num::NonZeroUsize, core::num::NonZeroU128,
    core::num::NonZeroI8, core::num::NonZeroI16, core::num::NonZeroI32, core::num::NonZeroI64, core::num::NonZeroIsize, core::num::NonZeroI128,
    core::num::Saturating<T> where <T: TypeSignature>,
    core::num::Wrapping<T> where <T: TypeSignature>,
    core::ops::Range<T> where <T: TypeSignature>,
    core::ops::RangeFrom<T> where <T: TypeSignature>,
    core::ops::RangeFull,
    core::ops::RangeInclusive<T> where <T: TypeSignature>,
    core::ops::RangeTo<T> where <T: TypeSignature>,
    core::ops::RangeToInclusive<T> where <T: TypeSignature>,
    core::pin::Pin<T> where <T: TypeSignature>,
    core::ptr::NonNull<T> where <T: TypeSignature>,
    core::sync::atomic::AtomicU8, core::sync::atomic::AtomicU16, core::sync::atomic::AtomicU32, core::sync::atomic::AtomicU64, core::sync::atomic::AtomicUsize,
    core::sync::atomic::AtomicI8, core::sync::atomic::AtomicI16, core::sync::atomic::AtomicI32, core::sync::atomic::AtomicI64, core::sync::atomic::AtomicIsize,
    core::time::Duration,
);

impl<const N: usize, T: TypeSignature> TypeSignature for [T; N] {
    const SIGNATURE: TypeSignatureHasher = TypeSignatureHasher {
        ty_name: "TypeSignature impl for [T; N]",
        ty_generics: &[&T::SIGNATURE],
        const_generic_hashes: &[__macro_export::hash_const_usize(N)],
        // Not formally correct, but good enough for stdlib types since they won't change
        variants: &[],
    };
}

/// Implement for a tuple of values which all implement [`TypeSignature`].
macro_rules! impl_for_tuple {
    ($(
        ( $( $elem_ty:ident, )* $(,)? )
    ),+ $(,)? ) => {$(

        impl< $( $elem_ty ),* > TypeSignature for ($($elem_ty,)*)
            where $( $elem_ty : TypeSignature ),*
        {
            const SIGNATURE: TypeSignatureHasher  = TypeSignatureHasher {
                ty_name: stringify!($crate::TypeSignature impl for ($( $elem_ty ),* ) ),
                ty_generics: &[
                    $( &<$elem_ty as $crate::TypeSignature>::SIGNATURE, )*
                ],
                const_generic_hashes: &[],
                // Not formally correct, but good enough for stdlib types since they won't change
                variants: &[],
            };
        }

    )+};
}

impl_for_tuple!(
    (T0,),
    (T0, T1,),
    (T0, T1, T2,),
    (T0, T1, T2, T3,),
    (T0, T1, T2, T3, T4,),
    (T0, T1, T2, T3, T4, T5,),
    (T0, T1, T2, T3, T4, T5, T6,),
    (T0, T1, T2, T3, T4, T5, T6, T7,),
    (T0, T1, T2, T3, T4, T5, T6, T7, T8,),
    (T0, T1, T2, T3, T4, T5, T6, T7, T8, T9,),
    (T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10,),
    (T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11,),
    (T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12,),
    (T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13,),
    (
        T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14,
    ),
    (
        T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15,
    ),
);

#[cfg(feature = "std")]
mod std_impl {
    extern crate std;

    use crate::TypeSignature;

    impl_for_stdlib_ty!(
        std::collections::HashMap<K, V> where <K: TypeSignature, V: TypeSignature>,
        std::collections::HashSet<T> where <T: TypeSignature>,
        std::ffi::OsStr,
        std::ffi::OsString,
        std::path::Path,
        std::path::PathBuf,
        std::time::Instant,
        std::time::SystemTime,
    );
}

#[cfg(feature = "alloc")]
mod alloc_impl {
    extern crate alloc;

    use crate::TypeSignature;

    impl_for_stdlib_ty!(
        alloc::boxed::Box<T> where <T: TypeSignature>,
        alloc::collections::BinaryHeap<T> where <T: TypeSignature>,
        alloc::collections::BTreeMap<K, V> where <K: TypeSignature, V: TypeSignature>,
        alloc::collections::BTreeSet<T> where <T: TypeSignature>,
        alloc::collections::LinkedList<T> where <T: TypeSignature>,
        alloc::collections::VecDeque<T> where <T: TypeSignature>,
        alloc::ffi::CString,
        alloc::rc::Rc<T> where <T: TypeSignature>,
        alloc::rc::Weak<T> where <T: TypeSignature>,
        alloc::string::String,
        alloc::sync::Arc<T> where <T: TypeSignature>,
        alloc::sync::Weak<T> where <T: TypeSignature>,
        alloc::vec::Vec<T> where <T: TypeSignature>,
    );

    impl<'a, B: TypeSignature + alloc::borrow::ToOwned + ?Sized + 'a> TypeSignature
        for alloc::borrow::Cow<'a, B>
    {
        const SIGNATURE: crate::TypeSignatureHasher = crate::TypeSignatureHasher {
            ty_name: "TypeSignature impl for Cow<'a, B>",
            ty_generics: &[&<B as TypeSignature>::SIGNATURE],
            const_generic_hashes: &[],
            variants: &[],
        };
    }
}

#[doc(hidden)]
pub mod __macro_export {
    /// Hash a const `usize` value.
    #[must_use]
    pub const fn hash_const_usize(param_val: usize) -> u64 {
        let mut accumulator = hash_str("usize");
        // TODO Better handle 128-bit targets
        mix_values(&mut accumulator, param_val as u64);
        accumulator
    }

    /// Hash a const `usize` value.
    #[must_use]
    pub const fn hash_const_bool(param_val: bool) -> u64 {
        let mut accumulator = hash_str("bool");
        mix_values(
            &mut accumulator,
            // Values chosen randomly to maximize number of bits different from any common pattern.
            if param_val {
                0x7907e475126f2049
            } else {
                0xa656facee66fd217
            },
        );
        accumulator
    }

    /// Mix a `u64` in to the accumulator.
    ///
    /// The mixing is done to ensure that the value is highly likely to change, and will likely
    /// be different for applying values in a different order.
    pub const fn mix_values(accumulator: &mut u64, value: u64) {
        // Constants are all primes, so multiplying and adding shuffles the values around
        // isomorphically.
        *accumulator = accumulator
            .wrapping_mul(0x35ce5fac9b4899b5)
            .wrapping_add(0x1e5d49b970ead075)
            ^ value
                .wrapping_mul(0x13fd608d551cc1d1)
                .wrapping_add(0x87b5240745caca0f);
    }

    /// Hash a string into a fixed `u64`.
    ///
    /// This function is designed to quickly jumble the contents, and result in vastly different
    /// hashes for even subtly-different strings.
    #[must_use]
    pub const fn hash_str(s: &str) -> u64 {
        let mut accumulator = 0x1124262e5999d5bb;
        let mut byte_idx = 0;
        while byte_idx < s.len() {
            mix_values(&mut accumulator, s.as_bytes()[byte_idx] as u64);
            byte_idx += 1;
        }
        accumulator
    }
}
