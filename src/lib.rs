#![forbid(unsafe_code)]
//! Macro for creating an enum where all variants have an associated constant string.
//! Syntax:
//! ```
//! str_enum::str_enum! {
//!     #[error_type(MyErrorType)] // Add this to opt-in to a FromStr implementation
//!     #[derive(Clone, Copy)] // You can add derives (exceptions: de/serialize enable the `serde` feature for that, Hash which is implemented automatically to be compatible with &str since the type is Borrow<str>)
//!     #[repr(u8)]
//!     pub enum MyEnum {
//!         Variant0 => "Value0"("other valid forms such as", "value0", "can go in brackets"), // note these other valid forms are only used in the enum's try_from_str method and FromStr implementation.
//!         Variant1 = 3 => "Value1" // you can add a discriminant
//!     }
//! }
//!
//! let var0 = MyEnum::try_from_str("value0").unwrap();
//! assert_eq!(var0.as_str(), MyEnum::Variant0.as_str())
//! ```
//!
//! Note, due to how we assemble some strings at compile time you'll see some constants that you likely never need to interact with.
//! You can just throw the enum in its own module to avoid seeing them since they're private visibility.

#[cfg(feature = "serde")]
pub use serde;

#[cfg(feature = "strum")]
pub use strum;

#[macro_export]
macro_rules! str_enum_base {
    ($(#[error_type($error_ty:ident)])? $(#[derive($($derive_trait:ident),* $(,)?)])? $(#[repr($repr:ty)])? $vis:vis enum $ty:ident { $($variant:ident $(= $variant_repr:literal)? => $val:literal $(($($other_valid:literal),* $(,)?))?),* $(,)? }) => {
        $(
            #[derive($($derive_trait,)*)]
        )?
        $(
            #[repr($repr)]
        )?
        $vis enum $ty {
            $(
                $variant $(= $variant_repr)?,
            )*
        }

        impl $ty {
            pub const ALL_VARIANTS: &[Self] = &[$(Self::$variant,)*];
            pub const NUM_VARIANTS: usize = Self::ALL_VARIANTS.len();

            pub const fn as_str(&self) -> &'static str {
                match self {
                    $(Self::$variant => $val,)*
                }
            }

            pub fn try_from_str(s: &str) -> Option<Self> {
                match s {
                    $($val $($(|$other_valid)*)? => Some(Self::$variant),)*
                    _ => None,
                }
            }

            pub const ALL_VALUES: &[&str] = &[$(Self::$variant.as_str(),)*];

            const ALL_VALUES_STR_LEN: usize = {
                let mut len = 0usize;
                let mut idx = 0usize;
                while idx < Self::ALL_VALUES.len() {
                    let value = Self::ALL_VALUES[idx];
                    len += value.len() + 1;
                    idx += 1
                }
                len - 1
            };

            const ALL_VALUE_BYTES: [u8; Self::ALL_VALUES_STR_LEN] = {
                let mut buf = [0u8; Self::ALL_VALUES_STR_LEN];
                let mut idx = 0;
                let mut buf_idx = 0;
                while idx < Self::ALL_VALUES.len() {
                    let value = Self::ALL_VALUES[idx];
                    let mut value_idx = 0;
                    while value_idx < value.len() {
                        buf[buf_idx] = value.as_bytes()[value_idx];
                        value_idx += 1;
                        buf_idx += 1
                    }

                    if idx != Self::ALL_VALUES.len() - 1 {
                        buf[buf_idx] = b',';
                        buf_idx += 1;
                    }
                    idx += 1
                }
                buf
            };

            // we assemble this in a funny way due to issues with slicing in const
            const ALL_VALUE_STR: &str = {
                match str::from_utf8(&Self::ALL_VALUE_BYTES) {
                    Ok(o) => o,
                    Err(_) => panic!(),
                }
            };
        }

        impl $ty {
            pub const fn len(&self) -> usize {
                self.as_str().len()
            }
        }

        $(
            impl $ty {
                fn into_repr(self) -> $repr {
                    self as $repr
                }
            }

            impl From<$ty> for $repr {
                fn from(v: $ty) -> $repr {
                    v as $repr
                }
            }
        )?

        impl std::fmt::Display for $ty {
            fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                <str as std::fmt::Display>::fmt(self.as_str(), fmt)
            }
        }

        impl std::borrow::Borrow<str> for $ty {
            fn borrow(&self) -> &str {
                self.as_str()
            }
        }

        impl std::hash::Hash for $ty {
            fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
                <str as std::hash::Hash>::hash(self.as_str(), state)
            }
        }

        impl<'a> std::ops::Add<$ty> for std::borrow::Cow<'a, str> {
            type Output = std::borrow::Cow<'a, str>;

            fn add(self, rhs: $ty) -> std::borrow::Cow<'a, str> {
                self.add(rhs.as_str())
            }
        }

        impl std::ops::Add<$ty> for String {
            type Output = String;

            fn add(self, rhs: $ty) -> String {
                self.add(rhs.as_str())
            }
        }

        impl<'a> std::ops::AddAssign<$ty> for std::borrow::Cow<'a, str> {
            fn add_assign(&mut self, rhs: $ty) {
                self.add_assign(rhs.as_str())
            }
        }

        impl std::ops::AddAssign<$ty> for String {
            fn add_assign(&mut self, rhs: $ty) {
                self.add_assign(rhs.as_str())
            }
        }

        $crate::str_enum_base!(AsRef $ty, [str, std::ffi::OsStr, std::path::Path, [u8]]);

        impl Extend<$ty> for String {
            fn extend<I>(&mut self, iter: I) where I: IntoIterator<Item = $ty> {
                iter.into_iter().for_each(move |s| self.push_str(s.as_str()))
            }
        }

        $crate::str_enum_base!(From $ty, [std::sync::Arc<str>, Box<str>, std::rc::Rc<str>, String, Vec<u8>]);
        $crate::str_enum_base!(From 'a $ty, [Box<dyn std::error::Error + 'a>, Box<dyn std::error::Error + Send + Sync + 'a>, std::borrow::Cow<'a, str>]);
        $crate::str_enum_base!(FromIterator $ty, [Box<str>, String]);
        $crate::str_enum_base!(FromIterator 'a $ty, [std::borrow::Cow<'a, str>]);

        impl<I: std::slice::SliceIndex<str>> std::ops::Index<I> for $ty {
            type Output = <I as std::slice::SliceIndex<str>>::Output;

            fn index(&self, index: I) -> &<I as std::slice::SliceIndex<str>>::Output {
                self.as_str().index(index)
            }
        }

        $crate::str_enum_base!(PartialEq $ty, [std::ffi::OsStr, std::ffi::OsString, String, std::path::Path, std::path::PathBuf]);
        $crate::str_enum_base!(PartialEq 'a $ty, [std::borrow::Cow<'a, str>]);

        impl PartialEq<&str> for $ty {
            fn eq(&self, rhs: &&str) -> bool {
                self.as_str().eq(*rhs)
            }
        }

        impl PartialEq<$ty> for &str {
            fn eq(&self, rhs: &$ty) -> bool {
                self.eq(&rhs.as_str())
            }
        }

        impl PartialEq<str> for $ty {
            fn eq(&self, rhs: &str) -> bool {
                self.as_str().eq(rhs)
            }
        }

        impl PartialEq<$ty> for str {
            fn eq(&self, rhs: &$ty) -> bool {
                self.eq(rhs.as_str())
            }
        }

        $crate::str_enum_base!(PartialOrd $ty, [std::ffi::OsStr, std::ffi::OsString]);

        impl PartialOrd<$ty> for str {
            fn partial_cmp(&self, rhs: &$ty) -> Option<std::cmp::Ordering> {
                self.partial_cmp(rhs.as_str())
            }
        }

        impl PartialOrd<str> for $ty {
            fn partial_cmp(&self, rhs: &str) -> Option<std::cmp::Ordering> {
                self.as_str().partial_cmp(rhs)
            }
        }

        impl PartialOrd<$ty> for &str {
            fn partial_cmp(&self, rhs: &$ty) -> Option<std::cmp::Ordering> {
                self.partial_cmp(&rhs.as_str())
            }
        }

        impl PartialOrd<&str> for $ty {
            fn partial_cmp(&self, rhs: &&str) -> Option<std::cmp::Ordering> {
                self.as_str().partial_cmp(*rhs)
            }
        }

        impl std::net::ToSocketAddrs for $ty {
            type Iter = std::vec::IntoIter<std::net::SocketAddr>;

            fn to_socket_addrs(&self) -> std::io::Result<std::vec::IntoIter<std::net::SocketAddr>> {
                <str as std::net::ToSocketAddrs>::to_socket_addrs(self.as_str())
            }
        }



        $(
            #[derive(Debug, Clone, Copy, Default)]
            $vis struct $error_ty;

            impl $error_ty {
                const EXPECTED_STR_LEN: usize = "expected one of [".len() + "]".len() + $ty::ALL_VALUES_STR_LEN;
                const EXPECTED_STR_BYTES: [u8; Self::EXPECTED_STR_LEN] = {
                    let mut buf = [0u8; Self::EXPECTED_STR_LEN];
                    let mut idx = 0;

                    let first_part = b"expected one of [";

                    while idx < first_part.len() {
                        buf[idx] = first_part[idx];
                        idx += 1
                    }

                    while idx < first_part.len() + $ty::ALL_VALUES_STR_LEN {
                        buf[idx] = $ty::ALL_VALUE_BYTES[idx - first_part.len()];
                        idx +=1
                    }
                    buf[Self::EXPECTED_STR_LEN - 1] = b']';

                    buf
                };

                const EXPECTED_STR: &str = {
                    match str::from_utf8(&Self::EXPECTED_STR_BYTES) {
                        Ok(o) => o,
                        Err(_) => panic!(),
                    }
                };
            }

            impl std::fmt::Display for $error_ty {
                fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    <str as std::fmt::Display>::fmt(Self::EXPECTED_STR, fmt)
                }
            }

            impl std::error::Error for $error_ty {}

            impl std::str::FromStr for $ty {
                type Err = $error_ty;

                fn from_str(s: &str) -> Result<$ty, Self::Err> {
                    match Self::try_from_str(s) {
                        Some(variant) => Ok(variant),
                        None => Err($error_ty)
                    }
                }
            }

            impl TryFrom<&str> for $ty {
                type Error = $error_ty;

                fn try_from(s: &str) -> Result<$ty, Self::Error> {
                    match Self::try_from_str(s) {
                        Some(variant) => Ok(variant),
                        None => Err($error_ty)
                    }
                }
            }

            impl TryFrom<String> for $ty {
                type Error = $error_ty;

                fn try_from(s: String) -> Result<$ty, Self::Error> {
                    match Self::try_from_str(&s) {
                        Some(variant) => Ok(variant),
                        None => Err($error_ty)
                    }
                }
            }

            impl<'a> TryFrom<&'a std::ffi::OsStr> for $ty {
                type Error = $crate::Utf8EnumError<$error_ty>;

                fn try_from(value: &'a std::ffi::OsStr) -> Result<$ty, Self::Error> {
                    <&'a str as TryFrom<&'a std::ffi::OsStr>>::try_from(value)
                    .map_err($crate::Utf8EnumError::Utf8)
                    .and_then(|s| $ty::try_from(s).map_err($crate::Utf8EnumError::InvalidVariant))
                }
            }
        )?
    };
    (AsRef $self:ident, [$($other:ty),*]) => {
        $(
            impl AsRef<$other> for $self {
                fn as_ref(&self) -> &$other {
                    <str as AsRef<$other>>::as_ref(self.as_str())
                }
            }
        )*
    };
    (From $self:ident, [$($other:ty),*]) => {
        $(
            impl From<$self> for $other {
                fn from(val: $self) -> $other {
                    From::from(val.as_str())
                }
            }
        )*
    };
    (From 'a $self:ident, [$($other:ty),*]) => {
        $(
            impl<'a> From<$self> for $other {
                fn from(val: $self) -> $other {
                    From::from(val.as_str())
                }
            }
        )*
    };
    (FromIterator $self:ident, [$($other:ty),*]) => {
        $(
            impl std::iter::FromIterator<$self> for $other {
                fn from_iter<T>(iter: T) -> $other
                where
                    T: IntoIterator<Item = $self>
                {
                    <$other as std::iter::FromIterator<&'static str>>::from_iter(iter.into_iter().map(|s| s.as_str()))
                }
            }
        )*
    };
    (FromIterator 'a $self:ident, [$($other:ty),*]) => {
        $(
            impl<'a> std::iter::FromIterator<$self> for $other {
                fn from_iter<T>(iter: T) -> $other
                where
                    T: IntoIterator<Item = $self>
                {
                    <$other as std::iter::FromIterator<&'static str>>::from_iter(iter.into_iter().map(|s| s.as_str()))
                }
            }
        )*
    };
    (PartialEq $self:ident, [$($other:ty),*]) => {
        $(
            impl PartialEq<$self> for $other {
                fn eq(&self, rhs: &$self) -> bool {
                    self.eq(rhs.as_str())
                }
            }

            impl PartialEq<$other> for $self {
                fn eq(&self, rhs: &$other) -> bool {
                    self.as_str().eq(rhs)
                }
            }
        )*
    };
    (PartialEq 'a $self:ident, [$($other:ty),*]) => {
        $(
            impl<'a> PartialEq<$self> for $other {
                fn eq(&self, rhs: &$self) -> bool {
                    self.eq(rhs.as_str())
                }
            }

            impl<'a> PartialEq<$other> for $self {
                fn eq(&self, rhs: &$other) -> bool {
                    self.as_str().eq(rhs)
                }
            }
        )*
    };
    (PartialOrd $self:ident, [$($other:ty),*]) => {
        $(
            impl PartialOrd<$self> for $other {
                fn partial_cmp(&self, rhs: &$self) -> Option<std::cmp::Ordering> {
                    self.partial_cmp(rhs.as_str())
                }
            }
        )*
    };
    (PartialOrd 'a $self:ident, [$($other:ty),*]) => {
        $(
            impl<'a> PartialOrd<$self> for $other {
                fn partial_cmp(&self, rhs: &$self) -> Option<std::cmp::Ordering> {
                    self.partial_cmp(rhs.as_str())
                }
            }
        )*
    };
}

#[cfg(feature = "strum")]
#[macro_export]
macro_rules! str_enum_strum {
    ($(#[error_type($error_ty:ident)])? $(#[derive($($derive_trait:ident),* $(,)?)])? $(#[repr($repr:ty)])? $vis:vis enum $ty:ident { $($variant:ident $(= $variant_repr:literal)? => $val:literal $(($($other_valid:literal),* $(,)?))?),* $(,)? }) => {
        impl $crate::strum::EnumCount for $ty {
            const COUNT: usize = $ty::ALL_VARIANTS.len();
        }

        impl $crate::strum::EnumProperty for $ty {
            fn get_str(&self, prop: &str) -> Option<&'static str> {
                Some(self.as_str())
            }

            fn get_int(&self, _: &str) -> Option<i64> {
                None
            }

            fn get_bool(&self, _: &str) -> Option<bool> {
                None
            }
        }

        $(
            impl $crate::strum::IntoDiscriminant for $ty {
                type Discriminant = $repr;

                fn discriminant(&self) -> Self::Discriminant {
                    self.into_repr()
                }
            }
        )?

        impl $crate::strum::IntoEnumIterator for $ty {
            type Iterator = std::array::IntoIter<$ty, {$ty::NUM_VARIANTS}>;

            fn iter() -> Self::Iterator {
                [$(Self::$variant,)*].into_iter()
            }
        }

        impl $crate::strum::VariantArray for $ty {
            const VARIANTS: &'static [Self] = Self::ALL_VARIANTS;
        }

        impl $crate::strum::VariantIterator for $ty {
            type Iterator = std::array::IntoIter<$ty, {$ty::NUM_VARIANTS}>;

            fn iter() -> Self::Iterator {
                [$(Self::$variant,)*].into_iter()
            }
        }

        impl $crate::strum::VariantNames for $ty {
            const VARIANTS: &'static [&'static str] = &[$(stringify!($variant),)*];
        }

        impl $crate::strum::VariantMetadata for $ty {
            const VARIANT_COUNT: usize = Self::ALL_VARIANTS.len();
            const VARIANT_NAMES: &'static [&'static str] = &[$(stringify!($variant),)*];

            fn variant_name(&self) -> &'static str {
                match self {
                    $(Self::$variant => stringify!($variant),)*
                }
            }
        }
    };
}

#[macro_export]
#[cfg(not(feature = "strum"))]
macro_rules! str_enum_strum {
    ($(#[error_type($error_ty:ident)])? $(#[derive($($derive_trait:ident),* $(,)?)])? $(#[repr($repr:ty)])? $vis:vis enum $ty:ident { $($variant:ident $(= $variant_repr:literal)? => $val:literal $(($($other_valid:literal),* $(,)?))?),* $(,)? }) => {};
}

#[macro_export]
#[cfg(feature = "serde")]
macro_rules! str_enum_serde {
    ($(#[error_type($error_ty:ident)])? $(#[derive($($derive_trait:ident),* $(,)?)])? $(#[repr($repr:ty)])? $vis:vis enum $ty:ident { $($variant:ident $(= $variant_repr:literal)? => $val:literal $(($($other_valid:literal),* $(,)?))?),* $(,)? }) => {
        impl $ty {
            const SERDE_EXPECTED_STR_LEN: usize = "one of [".len() + "]".len() + Self::ALL_VALUES_STR_LEN;
            const SERDE_EXPECTED_STR_BYTES: [u8; Self::SERDE_EXPECTED_STR_LEN] = {
                let mut buf = [0u8; Self::SERDE_EXPECTED_STR_LEN];
                let mut idx = 0;

                let first_part = b"one of [";

                while idx < first_part.len() {
                    buf[idx] = first_part[idx];
                    idx += 1
                }

                while idx < first_part.len() + Self::ALL_VALUES_STR_LEN {
                    buf[idx] = Self::ALL_VALUE_BYTES[idx - first_part.len()];
                    idx +=1
                }
                buf[Self::SERDE_EXPECTED_STR_LEN - 1] = b']';

                buf
            };

            const SERDE_EXPECTED_STR: &str = {
                match str::from_utf8(&Self::SERDE_EXPECTED_STR_BYTES) {
                    Ok(o) => o,
                    Err(_) => panic!(),
                }
            };
        }


        $(
            impl $crate::serde::de::Expected for $error_ty {
                fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                    <str as std::fmt::Display>::fmt($ty::SERDE_EXPECTED_STR, formatter)
                }
            }
        )?

        impl $crate::serde::Serialize for $ty {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: $crate::serde::Serializer,
            {
                self.as_str().serialize(serializer)
            }
        }

        impl<'de> $crate::serde::Deserialize<'de> for $ty {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: $crate::serde::Deserializer<'de>,
            {
                let val = <std::borrow::Cow<'_, str> as $crate::serde::Deserialize>::deserialize(deserializer)?;
                $ty::try_from_str(&val).ok_or_else(|| $crate::serde::de::Error::invalid_value($crate::serde::de::Unexpected::Str(&val), &$ty::SERDE_EXPECTED_STR))
            }
        }
    };
}

#[macro_export]
#[cfg(not(feature = "serde"))]
macro_rules! str_enum_serde {
    ($(#[error_type($error_ty:ident)])? $(#[derive($($derive_trait:ident),* $(,)?)])? $(#[repr($repr:ty)])? $vis:vis enum $ty:ident { $($variant:ident $(= $variant_repr:literal)? => $val:literal $(($($other_valid:literal),* $(,)?))?),* $(,)? }) => {};
}

#[macro_export]
macro_rules! str_enum {
    ($(#[error_type($error_ty:ident)])? $(#[derive($($derive_trait:ident),* $(,)?)])? $(#[repr($repr:ty)])? $vis:vis enum $ty:ident { $($variant:ident $(= $variant_repr:literal)? => $val:literal $(($($other_valid:literal),* $(,)?))?),* $(,)? }) => {
        $crate::str_enum_base!(
            $(#[error_type($error_ty)])?
            $(#[derive($($derive_trait,)*)])?
            $(#[repr($repr)])?
            $vis enum $ty {
                $($variant $(= $variant_repr)? => $val $(($($other_valid),*))?,)*
            }
        );

        $crate::str_enum_strum!(
            $(#[error_type($error_ty)])?
            $(#[derive($($derive_trait,)*)])?
            $(#[repr($repr)])?
            $vis enum $ty {
                $($variant $(= $variant_repr)? => $val $(($($other_valid),*))?,)*
            }
        );

        $crate::str_enum_serde!(
            $(#[error_type($error_ty)])?
            $(#[derive($($derive_trait,)*)])?
            $(#[repr($repr)])?
            $vis enum $ty {
                $($variant $(= $variant_repr)? => $val $(($($other_valid),*))?,)*
            }
        );

    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Utf8EnumError<E> {
    Utf8(std::str::Utf8Error),
    InvalidVariant(E),
}

impl<E> std::fmt::Display for Utf8EnumError<E>
where
    E: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Utf8EnumError::Utf8(utf8_error) => utf8_error.fmt(f),
            Utf8EnumError::InvalidVariant(variant_error) => variant_error.fmt(f),
        }
    }
}

impl<E> std::error::Error for Utf8EnumError<E> where E: std::error::Error {}
