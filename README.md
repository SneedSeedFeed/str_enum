# str_enum

A real simple declarative macro that I have ended up copy pasting into a few of my projects, spruced up with a couple features to make it more flexible.
Unless you're a real stickler for compile times you're probably fine with [`strum`](https://crates.io/crates/strum) over this for more features.

It implements basically everything `str` does, minus the derives which are opt-in. So you can probably use this anywhere you use &str.

```rust
use str_enum::str_enum;

str_enum! {
    #[error_type(MyError)] // optional: adds a FromStr implementation with the chosen error as the error type
    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)] // optional: adds the derives you specify to the enum. just not de/serialize, enable the serde feature for that
    #[repr(u8)] // optional: decide the repr
    pub(crate) enum MyEnum {
        Variant1 = 1 => "Variant1", // can optionally decide the discriminant for the variant
        Variant2 => "Variant2"("variant1"), // can add optional valid forms of input in brackets, if you want to cover lower case for example
    }
}
```

## Features

| feature | description |
| --- | --- |
| serde | Enables `serde` as a dependency and implements `Serialize` and `Deserialize` for the enum, respecting alternate valid forms. |
| strum | Enables `strum` as a dependency and implements `EnumCount`, `EnumProperty`, `IntoDiscriminant` (if you have a defined repr), `IntoEnumIterator`, `VariantArray`, `VariantIterator`, `VariantNames` and `VariantMetadata`. Note that the `IntoDiscriminant` implementation requires your enum to opt into implementing `Copy` |