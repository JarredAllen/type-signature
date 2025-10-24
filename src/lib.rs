#![no_std]

/// A type that can be made into a signature.
///
/// If implementing for a custom type, please use the derive macro instead.
pub trait TypeSignature {
    /// The signature of this type.
    const SIGNATURE: TypeSignatureHasher;
}

#[derive(Hash)]
pub struct TypeSignatureHasher {
    #[doc(hidden)]
    pub ty_name: &'static str,
    #[doc(hidden)]
    pub generics: &'static [&'static TypeSignatureHasher],
    #[doc(hidden)]
    pub fields: &'static [(&'static str, &'static TypeSignatureHasher)],
}
impl TypeSignatureHasher {
    pub const fn const_hash(&self) -> u64 {
        /// Mix a `u64` in to the accumulator.
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
            let mut field_idx = 0;
            while field_idx < self.fields.len() {
                let (field_name, field_hasher) = self.fields[field_idx];
                mix_values(&mut accumulator, hash_str(field_name));
                mix_values(&mut accumulator, field_hasher.const_hash());
                field_idx += 1;
            }
        }

        accumulator
    }
}

macro_rules! impl_for_stdlib_ty {
    ($(
        $stdty:ty $(where < $( $generic:ident $( : $generic_cond:tt )? ),* > )?
    ),+ $(,)?) => {$(
        impl$( < $( $generic $( : $generic_cond )? ),* > )? $crate::TypeSignature for $stdty {
            const SIGNATURE: TypeSignatureHasher  = TypeSignatureHasher {
                ty_name: stringify!($crate::TypeSignature impl for $stdty),
                generics: &[
                    $( $( &<$generic as $crate::TypeSignature>::SIGNATURE, )* )?
                ],
                // Not formally correct, but good enough for stdlib types since they won't change
                fields: &[],
            };
        }
    )+};
}

impl_for_stdlib_ty!(
    u8,
    u16,
    u32,
    usize,
    u64,
    u128,
    i8,
    i16,
    i32,
    isize,
    i64,
    i128,
    bool,
    f32,
    f64,
    char,
    str,
    (),
    [T] where <T: TypeSignature>,
    Option<T> where <T: TypeSignature> ,
    Result<T, E> where <T: TypeSignature, E: TypeSignature>,
);

impl<const N: usize, T: TypeSignature> TypeSignature for [T; N] {
    const SIGNATURE: TypeSignatureHasher = TypeSignatureHasher {
        ty_name: "TypeSignature impl for [T; N]",
        generics: &[&T::SIGNATURE],
        // Not formally correct, but good enough for stdlib types since they won't change
        fields: &[],
    };
}

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
                fields: &[],
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
