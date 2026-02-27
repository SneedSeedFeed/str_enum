# str_enum

A real simple declarative macro that I have ended up copy pasting into a few of my projects, spruced up with a couple features to make it more flexible.
Unless you're a real stickler for compile times you're probably fine with [`strum`](https://crates.io/crates/strum) over this for more features.

It implements basically everything `str` does, minus the derives which are opt-in. So you can probably use this anywhere you use &str.

```rust
use str_enum::str_enum;

str_enum! {
    #[error_type(MyError)] // optional: adds a FromStr implementation with the chosen error as the error type
    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)] // optional: adds the derives you specify to the enum. just not de/serialize, enable the serde feature for that
    pub(crate) enum MyEnum {
        Variant1 = "Variant1"("variant1"), // can add optional valid forms of input in brackets, if you want to cover lower case 
        Variant2 = "Variant2",
    }
}
```