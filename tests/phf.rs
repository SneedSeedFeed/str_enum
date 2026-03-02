#![cfg(feature = "phf")]

use str_enum::str_enum;

str_enum! {
    #[error_type(MyError)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    #[repr(u8)]
    pub(crate) enum MyEnum {
        Variant1 = 5 => "Variant1"("variant1"),
        Variant2 => "Variant2",
    }
}

str_enum! {
    #[phf]
    #[error_type(PhfError)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    #[repr(u8)]
    pub(crate) enum PhfEnum {
        Variant1 = 5 => "Variant1"("variant1"),
        Variant2 => "Variant2",
    }
}

#[test]
fn test_phf_has_map() {
    assert!(!PhfEnum::PHF_MAP.is_empty())
}

#[test]
fn test_phf_try_from_str_primary() {
    assert_eq!(PhfEnum::try_from_str("Variant1"), Some(PhfEnum::Variant1));
    assert_eq!(PhfEnum::try_from_str("Variant2"), Some(PhfEnum::Variant2));
}

#[test]
fn test_phf_try_from_str_alternate() {
    assert_eq!(PhfEnum::try_from_str("variant1"), Some(PhfEnum::Variant1));
}

#[test]
fn test_phf_try_from_str_invalid() {
    assert_eq!(PhfEnum::try_from_str("nonexistent"), None);
}

#[test]
fn test_phf_from_str_primary() {
    let v1: PhfEnum = "Variant1".parse().unwrap();
    let v2: PhfEnum = "Variant2".parse().unwrap();
    assert_eq!(v1, PhfEnum::Variant1);
    assert_eq!(v2, PhfEnum::Variant2);
}

#[test]
fn test_phf_from_str_alternate() {
    let v1: PhfEnum = "variant1".parse().unwrap();
    assert_eq!(v1, PhfEnum::Variant1);
}

#[test]
fn test_phf_from_str_invalid() {
    let result: Result<PhfEnum, _> = "nonexistent".parse();
    assert!(result.is_err());
}

#[test]
fn test_phf_and_match_agree_on_all_primary_values() {
    for val in MyEnum::ALL_VALUES {
        let match_result = MyEnum::try_from_str(val);
        let phf_result = PhfEnum::try_from_str(val);
        assert_eq!(
            match_result.is_some(),
            phf_result.is_some(),
            "disagreement on primary value {val:?}"
        );
        assert_eq!(
            match_result.map(|v| v.as_str()),
            phf_result.map(|v| v.as_str()),
            "different variant resolved for {val:?}"
        );
    }
}

#[test]
fn test_phf_and_match_agree_on_alternates() {
    let alternate_inputs = ["variant1"];
    for val in alternate_inputs {
        let match_result = MyEnum::try_from_str(val);
        let phf_result = PhfEnum::try_from_str(val);
        assert_eq!(
            match_result.map(|v| v.as_str()),
            phf_result.map(|v| v.as_str()),
            "disagreement on alternate value {val:?}"
        );
    }
}

#[test]
fn test_phf_and_match_agree_on_invalid() {
    let invalid_inputs = ["nonexistent", "", "VARIANT1", "Variant3", " Variant1"];
    for val in invalid_inputs {
        assert_eq!(
            MyEnum::try_from_str(val).is_none(),
            PhfEnum::try_from_str(val).is_none(),
            "disagreement on invalid value {val:?}"
        );
    }
}

#[test]
fn test_phf_try_from_trait() {
    let v1 = PhfEnum::try_from("Variant1").unwrap();
    assert_eq!(v1, PhfEnum::Variant1);

    let v1_alt = PhfEnum::try_from("variant1").unwrap();
    assert_eq!(v1_alt, PhfEnum::Variant1);

    assert!(PhfEnum::try_from("nonexistent").is_err());
}

#[test]
fn test_phf_try_from_string() {
    let v1 = PhfEnum::try_from(String::from("Variant1")).unwrap();
    assert_eq!(v1, PhfEnum::Variant1);

    assert!(PhfEnum::try_from(String::from("nonexistent")).is_err());
}

#[test]
fn test_phf_try_from_os_str() {
    use std::ffi::OsStr;
    let v1 = PhfEnum::try_from(OsStr::new("Variant1")).unwrap();
    assert_eq!(v1, PhfEnum::Variant1);

    assert!(PhfEnum::try_from(OsStr::new("nonexistent")).is_err());
}

#[test]
fn test_non_phf_still_works_with_feature_enabled() {
    assert_eq!(MyEnum::try_from_str("Variant1"), Some(MyEnum::Variant1));
    assert_eq!(MyEnum::try_from_str("variant1"), Some(MyEnum::Variant1));
    assert_eq!(MyEnum::try_from_str("Variant2"), Some(MyEnum::Variant2));
    assert_eq!(MyEnum::try_from_str("nonexistent"), None);
}

#[test]
fn test_non_phf_from_str_with_feature_enabled() {
    let v1: MyEnum = "Variant1".parse().unwrap();
    assert_eq!(v1, MyEnum::Variant1);
    let result: Result<MyEnum, _> = "nonexistent".parse();
    assert!(result.is_err());
}

#[cfg(feature = "serde")]
mod phf_serde {
    use super::PhfEnum;

    #[test]
    fn test_phf_serialize() {
        assert_eq!(
            serde_json::to_string(&PhfEnum::Variant1).unwrap(),
            "\"Variant1\""
        );
        assert_eq!(
            serde_json::to_string(&PhfEnum::Variant2).unwrap(),
            "\"Variant2\""
        );
    }

    #[test]
    fn test_phf_deserialize_primary() {
        let v1: PhfEnum = serde_json::from_str("\"Variant1\"").unwrap();
        assert_eq!(v1, PhfEnum::Variant1);
    }

    #[test]
    fn test_phf_deserialize_alternate() {
        let v1: PhfEnum = serde_json::from_str("\"variant1\"").unwrap();
        assert_eq!(v1, PhfEnum::Variant1);
    }

    #[test]
    fn test_phf_deserialize_invalid() {
        let result: Result<PhfEnum, _> = serde_json::from_str("\"nonexistent\"");
        assert!(result.is_err());
    }

    #[test]
    fn test_phf_serde_roundtrip() {
        for variant in PhfEnum::ALL_VARIANTS {
            let serialized = serde_json::to_string(variant).unwrap();
            let deserialized: PhfEnum = serde_json::from_str(&serialized).unwrap();
            assert_eq!(*variant, deserialized);
        }
    }
}
