#[macro_export]
macro_rules! assert_success {
    ($res:expr) => {
        assert!(
            $res.exit_code == 0,
            "expected exit code 0, got {}:\nstdout: {}\nstderr: {}",
            $res.exit_code,
            $res.stdout,
            $res.stderr
        );
    };
}

#[macro_export]
macro_rules! assert_contains {
    ($haystack:expr, $needle:expr) => {{
        let haystack = &$haystack;
        let needle = &$needle;
        assert!(
            haystack.contains(needle),
            "expected output to contain {:?}\ngot: {:?}",
            needle,
            haystack
        );
    }};
}
