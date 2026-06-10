use serde::{Serialize, de::DeserializeOwned};
use std::{collections::BTreeSet, fmt::Display, str::FromStr};

pub fn assert_public_name_contract<T>(values: &[T])
where
    T: Copy + Display + FromStr + Eq + std::fmt::Debug + Serialize + DeserializeOwned,
    <T as FromStr>::Err: std::fmt::Debug,
{
    assert_display_names_are_unique(values);
    assert_round_trip(values);
    assert_json_names_match_display(values);
    assert_rejects_surrounding_whitespace(values);
}

pub fn assert_display_names_are_unique<T>(values: &[T])
where
    T: Copy + Display,
{
    let names = values
        .iter()
        .map(ToString::to_string)
        .collect::<BTreeSet<_>>();

    assert_eq!(values.len(), names.len());
}

pub fn assert_round_trip<T>(values: &[T])
where
    T: Copy + Display + FromStr + Eq + std::fmt::Debug,
    <T as FromStr>::Err: std::fmt::Debug,
{
    for value in values {
        let parsed = T::from_str(&value.to_string()).expect("display name must parse");
        assert_eq!(*value, parsed);
    }
}

pub fn assert_rejects<T>(invalid_names: &[&str])
where
    T: FromStr,
{
    for invalid_name in invalid_names {
        assert!(
            T::from_str(invalid_name).is_err(),
            "accepted {invalid_name:?}"
        );
    }
}

fn assert_json_names_match_display<T>(values: &[T])
where
    T: Copy + Display + Eq + std::fmt::Debug + Serialize + DeserializeOwned,
{
    for value in values {
        let encoded = serde_json::to_string(value).expect("json encode");
        let decoded: T = serde_json::from_str(&encoded).expect("json decode");

        assert_eq!(format!("\"{value}\""), encoded);
        assert_eq!(*value, decoded);
    }
}

fn assert_rejects_surrounding_whitespace<T>(values: &[T])
where
    T: Copy + Display + FromStr,
{
    for value in values {
        let display_name = value.to_string();
        for invalid_name in [
            format!(" {display_name}"),
            format!("{display_name} "),
            format!("\t{display_name}"),
            format!("{display_name}\n"),
        ] {
            assert!(
                T::from_str(&invalid_name).is_err(),
                "accepted whitespace-padded name {invalid_name:?}"
            );
        }
    }
}
