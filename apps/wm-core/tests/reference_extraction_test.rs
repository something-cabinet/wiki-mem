use wm_core::reference::extract_references;

#[test]
fn sentence_final_reference_drops_the_trailing_period() {
    let refs = extract_references("Violates @wiki/rules/durable-write-or-error.");
    assert_eq!(refs.len(), 1);
    assert_eq!(refs[0].target, "durable-write-or-error");
    assert_eq!(refs[0].full_match, "@wiki/rules/durable-write-or-error");
}

#[test]
fn several_trailing_periods_are_all_dropped() {
    let refs = extract_references("Trailing off @wiki/tasks/fix-auth...");
    assert_eq!(refs.len(), 1);
    assert_eq!(refs[0].target, "fix-auth");
}

#[test]
fn comma_followed_reference_keeps_its_name() {
    let refs = extract_references("See @wiki/tasks/fix-auth, then merge.");
    assert_eq!(refs.len(), 1);
    assert_eq!(refs[0].target, "fix-auth");
}

#[test]
fn parenthesised_reference_keeps_its_name() {
    let refs = extract_references("Done (@wiki/tasks/fix-auth).");
    assert_eq!(refs.len(), 1);
    assert_eq!(refs[0].target, "fix-auth");
}

#[test]
fn period_inside_a_name_is_kept() {
    let refs =
        extract_references("Config in @wiki/reference/config.v2 and @wiki/reference/config.v2.");
    assert_eq!(refs.len(), 2);
    assert_eq!(refs[0].target, "config.v2");
    assert_eq!(refs[1].target, "config.v2");
}

#[test]
fn reference_made_only_of_periods_is_ignored() {
    let refs = extract_references("Odd @wiki/tasks/... reference");
    assert!(refs.is_empty());
}
