use std::{collections::HashMap, hash::DefaultHasher};

use str_enum::str_enum;

str_enum! {
    #[error_type(MyError)]
    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
    pub(crate) enum MyEnum {
        Variant1 = "Variant1"("variant1"),
        Variant2 = "Variant2",
    }
}

#[test]
fn test_from_str_primary() {
    let v1: MyEnum = "Variant1".parse().unwrap();
    let v2: MyEnum = "Variant2".parse().unwrap();
    assert_eq!(v1, MyEnum::Variant1);
    assert_eq!(v2, MyEnum::Variant2);
}

#[test]
fn test_from_str_alternate() {
    let v1: MyEnum = "variant1".parse().unwrap();
    assert_eq!(v1, MyEnum::Variant1);
}

#[test]
fn test_from_str_invalid() {
    let result: Result<MyEnum, _> = "nonexistent".parse();
    assert!(result.is_err());
}

#[test]
fn test_debug() {
    assert_eq!(format!("{:?}", MyEnum::Variant1), "Variant1");
    assert_eq!(format!("{:?}", MyEnum::Variant2), "Variant2");
}

#[test]
fn test_clone() {
    let v1 = MyEnum::Variant1;
    let v1_clone = v1.clone();
    assert_eq!(v1, v1_clone);
}

#[test]
fn test_ordering() {
    assert!(MyEnum::Variant1 < MyEnum::Variant2);
    let mut variants = vec![MyEnum::Variant2, MyEnum::Variant1];
    variants.sort();
    assert_eq!(variants, vec![MyEnum::Variant1, MyEnum::Variant2]);
}

#[test]
fn test_all_values_str() {
    assert_eq!(MyEnum::ALL_VALUE_STR, "Variant1,Variant2");
}

#[test]
fn test_error_expected_str() {
    assert_eq!(MyError::EXPECTED_STR, "expected one of [Variant1,Variant2]");
}

#[test]
fn test_display() {
    assert_eq!(format!("{}", MyEnum::Variant1), "Variant1");
    assert_eq!(format!("{}", MyEnum::Variant2), "Variant2");
}

#[test]
fn test_error_display() {
    let err = MyError;
    assert_eq!(format!("{err}"), "expected one of [Variant1,Variant2]");
}

#[test]
fn test_hashmap_str_lookup() {
    let mut map = HashMap::<MyEnum, u32>::new();
    map.insert(MyEnum::Variant1, 1);
    map.insert(MyEnum::Variant2, 2);

    assert_eq!(map.get("Variant1"), Some(&1));
    assert_eq!(map.get("Variant2"), Some(&2));
    assert_eq!(map.get(&MyEnum::Variant1), Some(&1));
    assert_eq!(map.get(&MyEnum::Variant2), Some(&2));
    assert_eq!(map.get("variant1"), None);
}

fn hash_of<T: std::hash::Hash>(t: &T) -> u64 {
    let mut hasher = DefaultHasher::new();
    t.hash(&mut hasher);
    std::hash::Hasher::finish(&hasher)
}

#[test]
fn test_hash_different_variants_differ() {
    assert_ne!(hash_of(&MyEnum::Variant1), hash_of(&MyEnum::Variant2));
}

#[cfg(feature = "serde")]
mod serde {
    use crate::MyEnum;

    #[test]
    fn test_serialize() {
        let v1 = MyEnum::Variant1;
        let v2 = MyEnum::Variant2;
        assert_eq!(serde_json::to_string(&v1).unwrap(), "\"Variant1\"");
        assert_eq!(serde_json::to_string(&v2).unwrap(), "\"Variant2\"");
    }

    #[test]
    fn test_deserialize_primary() {
        let v1: MyEnum = serde_json::from_str("\"Variant1\"").unwrap();
        let v2: MyEnum = serde_json::from_str("\"Variant2\"").unwrap();
        assert_eq!(v1, MyEnum::Variant1);
        assert_eq!(v2, MyEnum::Variant2);
    }

    #[test]
    fn test_deserialize_alternate() {
        let v1: MyEnum = serde_json::from_str("\"variant1\"").unwrap();
        assert_eq!(v1, MyEnum::Variant1);
    }

    #[test]
    fn test_deserialize_invalid() {
        let result: Result<MyEnum, _> = serde_json::from_str("\"nonexistent\"");
        assert!(result.is_err());
    }

    #[test]
    fn test_serde_roundtrip() {
        for variant in MyEnum::ALL_VARIANTS {
            let serialized = serde_json::to_string(variant).unwrap();
            let deserialized: MyEnum = serde_json::from_str(&serialized).unwrap();
            assert_eq!(*variant, deserialized);
        }
    }

    #[test]
    fn test_serde_expected_str() {
        assert_eq!(MyEnum::SERDE_EXPECTED_STR, "one of [Variant1,Variant2]");
    }
}
