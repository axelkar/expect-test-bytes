use super::UPDATE_EXPECT_VAR_NAME;
use std::{fs, sync::RwLock};

/// Makes tests that modify environment variables run independently.
static ENVVAR_MUTATION: RwLock<()> = RwLock::new(());

#[test]
fn succeeds() {
    let _guard = ENVVAR_MUTATION.read().unwrap();
    let expect = expect_file!["test_data/example"];
    expect.assert_eq(b"example\n");
}

#[test]
fn fails_missing() {
    let actual = {
        let _guard = ENVVAR_MUTATION.read().unwrap();
        let expect = expect_file!["test_data/missing"];

        let mut buf = Vec::new();
        assert!(expect.assert_eq_nopanic_imp(b"example\n", &mut buf).is_err());
        String::from_utf8(buf).expect("Only printing strings")
    };

    expect_test::expect_file!["test_data/fails_missing.ansi.bin"].assert_eq(&actual);
}

#[test]
fn fails_different() {
    let actual = {
        let _guard = ENVVAR_MUTATION.read().unwrap();
        let expect = expect_file!["test_data/example"];

        let mut buf = Vec::new();
        assert!(expect.assert_eq_nopanic_imp(b"exa- not this\n", &mut buf).is_err());
        String::from_utf8(buf).expect("Only printing strings")
    };

    expect_test::expect_file!["test_data/fails_different.ansi.bin"].assert_eq(&actual);
}

#[test]
fn creates() {
    let actual = {
        let _guard = ENVVAR_MUTATION.write().unwrap();
        std::env::set_var(UPDATE_EXPECT_VAR_NAME, "");

        let expect = expect_file!["test_data/creates"];

        let mut buf = Vec::new();
        assert!(expect.assert_eq_nopanic_imp(b"example\n", &mut buf).is_ok());

        // Not public API!
        fs::remove_file(&expect.path).unwrap();

        std::env::remove_var(UPDATE_EXPECT_VAR_NAME);

        String::from_utf8(buf).expect("Only printing strings")
    };

    expect_test::expect_file!["test_data/creates.ansi.bin"].assert_eq(&actual);
}
