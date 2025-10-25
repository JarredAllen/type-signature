#![no_std]

/// A type that can be made into a signature.
///
/// If implementing for a custom type, please use the derive macro instead.
pub trait TypeSignature {
    /// The signature of this type.
    const SIGNATURE: TypeSignatureHasher;

    /// A const-available u64 value.
    const CONST_HASH: u64 = Self::SIGNATURE.const_hash();
}

/// A hashable type for generating a signature for a type.
#[derive(Hash)]
pub struct TypeSignatureHasher {
    /// The name of the type being hashed.
    #[doc(hidden)]
    pub ty_name: &'static str,
    /// The types of the generic arguments.
    #[doc(hidden)]
    pub generics: &'static [&'static TypeSignatureHasher],
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
    /// This function exists to cover for the inability to call [`Hash::hash`] at const-time, and
    /// will likely be deprecated once const traits exist.
    pub const fn const_hash(&self) -> u64 {
        /// Mix a `u64` in to the accumulator.
        ///
        /// The mixing is done to ensure that the value is highly likely to change, and will likely
        /// be different for applying values in a different order.
        const fn mix_values(accumulator: &mut u64, value: u64) {
            // Constants are all primes, so multiplying and adding shuffles the values around
            // isomorphically.
            *accumulator = accumulator
                .wrapping_mul(0x35ce5fac9b4899b5)
                .wrapping_add(0x1e5d49b970ead075)
                | value
                    .wrapping_mul(0x13fd608d551cc1d1)
                    .wrapping_add(0x87b5240745caca0f);
        }

        /// Hash a string into a fixed `u64`.
        ///
        /// This function is designed to quickly jumble the contents
        const fn hash_str(s: &str) -> u64 {
            let mut accumulator = 0x1124262e5999d5bb;
            let mut byte_idx = 0;
            while byte_idx < s.len() {
                mix_values(&mut accumulator, s.as_bytes()[byte_idx] as u64);
                byte_idx += 1;
            }
            accumulator
        }

        let mut accumulator = 0x1b6142dc880364ed;

        // Mix in the name of the type
        mix_values(&mut accumulator, hash_str(self.ty_name));

        // Mix in the types of each generic.
        {
            let mut generic_idx = 0;
            while generic_idx < self.generics.len() {
                mix_values(&mut accumulator, self.generics[generic_idx].const_hash());
                generic_idx += 1;
            }
        }

        // Mix in the types and names of each field.
        {
            let mut variant_idx = 0;
            while variant_idx < self.variants.len() {
                let (variant_name, variant_fields) = self.variants[variant_idx];
                mix_values(&mut accumulator, hash_str(variant_name));
                let mut field_idx = 0;
                while field_idx < variant_fields.len() {
                    let (field_name, field_hasher) = variant_fields[field_idx];
                    mix_values(&mut accumulator, hash_str(field_name));
                    mix_values(&mut accumulator, field_hasher.const_hash());
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
                generics: &[
                    $( $( &<$generic as $crate::TypeSignature>::SIGNATURE, )* )?
                ],
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
        generics: &[&T::SIGNATURE],
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
                generics: &[
                    $( &<$elem_ty as $crate::TypeSignature>::SIGNATURE, )*
                ],
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
    (T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14,),
    (T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15,),
);

#[cfg(feature = "std")]
mod std_impl {
    extern crate std;

    use crate::TypeSignature;

    impl_for_stdlib_ty!(
        std::collections::HashMap<K, V> where <K: TypeSignature, V: TypeSignature>,
        std::collections::HashSet<T> where <T: TypeSignature>,
        std::sync::Arc<T> where <T: TypeSignature>,
        std::sync::Weak<T> where <T: TypeSignature>,
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
        alloc::rc::Rc<T> where <T: TypeSignature>,
        alloc::rc::Weak<T> where <T: TypeSignature>,
        alloc::vec::Vec<T> where <T: TypeSignature>,
    );
}
