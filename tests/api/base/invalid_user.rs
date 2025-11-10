#![allow(unused)]
use chrono::{Datelike, NaiveDate, Utc};
use proptest::prelude::*;

// Name
fn invalid_name() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("".to_string()),
        Just(" ".to_string()),
        Just("  ".to_string()),
        Just("\t".to_string()),
        Just("\n".to_string())
    ]
}

fn valid_name() -> impl Strategy<Value = String> {
    "[A-Z][a-z]{2,15}".prop_map(|s| s.to_string())
}

// Email

fn invalid_email() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("".to_string()),
        Just(" ".to_string()),
        Just("not-an-email".to_string()),
        Just("missing-at-sign.com".to_string()),
        Just("@no-local-part.com".to_string()),
        Just("no-domain@".to_string()),
        Just("spaces in@email.com".to_string()),
        Just("double@@email.com".to_string()),
    ]
}

fn valid_email() -> impl Strategy<Value = String> {
    "[a-z]{3,10}@[a-z]{3,10}\\.(com|org|net)".prop_map(|s| s.to_string())
}

// Password
fn too_short_password() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("".to_string()),
        Just("a".to_string()),
        Just("12".to_string()),
        Just("abc".to_string()),
        Just("1234".to_string()),
        Just("pass".to_string()),
        Just("1234567".to_string()),
        "[a-zA-Z0-9]{1,7}".prop_map(|s| s.to_string()),
    ]
}

fn too_long_password() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9!@#$%^&*]{129,200}".prop_map(|s| s.to_string())
}

fn valid_password() -> impl Strategy<Value = String> {
    // Password with: 8-128 chars, uppercase, lowercase, number, special char
    "[a-z]{2,5}[A-Z]{2,5}[0-9]{2,5}[!@#$%^&*]{1,3}[a-zA-Z0-9!@#$%^&*]{0,10}"
        .prop_map(|s| s.to_string())
}

fn invalid_password() -> impl Strategy<Value = String> {
    prop_oneof![too_short_password()]
}

// Username
fn too_long_username() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9!@#$%^&*]{256,300}".prop_map(|s| s.to_string())
}

fn only_special_chars_username() -> impl Strategy<Value = String> {
    "[!@#$%^&*()_+\\-=\\[\\]{}|;:,.<>?]{8,20}".prop_map(|s| s.to_string())
}

fn invalid_username() -> impl Strategy<Value = String> {
    prop_oneof![
        too_long_username(),
        only_special_chars_username(),
        Just("".to_string())
    ]
}

// DOB
fn underage_dob() -> impl Strategy<Value = NaiveDate> {
    let today = Utc::now().naive_utc().date();
    let eighteen_yrs_ago = (today - today.with_year(today.year() - 18).unwrap()).num_days();

    let min_days_ago = 1;

    (min_days_ago..eighteen_yrs_ago).prop_map(move |days| today - chrono::Duration::days(days))
}

fn valid_dob() -> impl Strategy<Value = NaiveDate> {
    let today = Utc::now().naive_utc().date();
    let eighteen_yrs_ago = (today - today.with_year(today.year() - 18).unwrap()).num_days();

    (eighteen_yrs_ago..120 * 365).prop_map(move |days| today - chrono::Duration::days(days))
}

pub fn create_underage_user(role: &str) -> impl Strategy<Value = serde_json::Value> {
    prop_oneof![
        (
            invalid_name(),
            invalid_name(),
            underage_dob(),
            invalid_username(),
            invalid_password(),
            invalid_email()
        )
            .prop_map(move |(f, l, d, u, p, e)| serde_json::json!({"first_name": f, "last_name": l, "date_of_birth": d, "username": u, "password": p, "email": e, "access_role": role}))
    ]
}

pub fn create_invalid_user(role: &str) -> impl Strategy<Value = serde_json::Value> {
    prop_oneof![
        (
            invalid_name(),
            invalid_name(),
            valid_dob(),
            invalid_username(),
            invalid_password(),
            invalid_email()
        )
            .prop_map(move |(f, l, d, u, p, e)| serde_json::json!({"first_name": f, "last_name": l, "date_of_birth": d, "username": u, "password": p, "email": e, "access_role": role}))
    ]
}
