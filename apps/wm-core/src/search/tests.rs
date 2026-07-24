#[cfg(test)]
mod tests {
    use crate::config::RecencyModel;
    use wm_search::{tokenize, Bm25Index, Field, IndexedDoc, cap_total_boost, recency_boost};

    fn make_test_index() -> Bm25Index {
        let docs = vec![
            IndexedDoc {
                id: "wiki:concepts:auth".into(),
                fields: vec![
                    Field::new("title", "Authentication Architecture", 4.0),
                    Field::new("tags", "auth security oauth2", 2.2),
                    Field::new("body", "JWT tokens with RS256 signing", 1.0),
                ],
            },
            IndexedDoc {
                id: "wiki:patterns:oauth2".into(),
                fields: vec![
                    Field::new("title", "OAuth2 Authorization Flow", 4.0),
                    Field::new("tags", "auth oauth2 security", 2.2),
                    Field::new("body", "Authorization code grant with PKCE extension", 1.0),
                ],
            },
            IndexedDoc {
                id: "wiki:reference:errors".into(),
                fields: vec![
                    Field::new("title", "Error Codes Reference", 4.0),
                    Field::new("tags", "errors reference", 2.2),
                    Field::new(
                        "body",
                        "ERR_AUTH_401: token expired, ERR_AUTH_403: forbidden",
                        1.0,
                    ),
                ],
            },
        ];
        Bm25Index::build(docs)
    }

    #[test]
    fn test_field_weighted_scoring() {
        let index = make_test_index();
        let results = index.search("authentication", 10);
        assert!(!results.is_empty());
        assert_eq!(results[0].id, "wiki:concepts:auth");
    }

    #[test]
    fn test_code_aware_tokenizer() {
        let tokens = tokenize("ERR_AUTH_401");
        assert!(tokens.contains(&"err_auth_401".to_string()));
        assert!(tokens.contains(&"err".to_string()));
        assert!(tokens.contains(&"auth".to_string()));
        assert!(tokens.contains(&"401".to_string()));
    }

    #[test]
    fn test_search_finds_error_code() {
        let index = make_test_index();
        let results = index.search("ERR_AUTH_401", 10);
        assert!(!results.is_empty());
        assert_eq!(results[0].id, "wiki:reference:errors");
    }

    #[test]
    fn test_score_normalization() {
        let index = make_test_index();
        let results = index.search("oauth2", 10);
        assert!(!results.is_empty(), "oauth2 should match the OAuth2 page");
        for r in &results {
            assert!(
                r.score >= 0.0 && r.score <= 1.0,
                "Score {} out of range for {}",
                r.score,
                r.id
            );
        }
    }

    #[test]
    fn test_tokenize_empty() {
        let tokens = tokenize("");
        assert!(tokens.is_empty());
    }

    #[test]
    fn test_tokenize_kebab_case() {
        let tokens = tokenize("auth-service");
        assert!(tokens.contains(&"auth-service".to_string()));
        assert!(tokens.contains(&"auth".to_string()));
        assert!(tokens.contains(&"service".to_string()));
    }

    #[test]
    fn test_zero_result_guard() {
        let index = make_test_index();
        let results = index.search("xyznonexistent123!!!", 10);
        assert!(results.is_empty());
    }

    #[test]
    fn test_recency_boost_fsrs_day0() {
        let b = recency_boost(0.0, &RecencyModel::Fsrs, 7.0);
        assert!((b - 1.0).abs() < 1e-6, "Day 0 should be 1.0, got {b}");
    }

    #[test]
    fn test_recency_boost_fsrs_day7() {
        let b = recency_boost(7.0, &RecencyModel::Fsrs, 7.0);
        assert!((b - 0.9).abs() < 0.01, "Day 7 (t=S) should be ~0.9, got {b}");
    }

    #[test]
    fn test_recency_boost_fsrs_day30() {
        let b = recency_boost(30.0, &RecencyModel::Fsrs, 7.0);
        assert!(b > 0.6 && b < 0.9, "Day 30 S=7 should be ~0.78, got {b}");
    }

    #[test]
    fn test_recency_boost_linear() {
        assert!((recency_boost(0.0, &RecencyModel::Linear, 7.0) - 1.0).abs() < 1e-6);
        assert!((recency_boost(7.0, &RecencyModel::Linear, 7.0) - 0.0).abs() < 1e-6);
        assert!((recency_boost(3.5, &RecencyModel::Linear, 7.0) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_recency_boost_exponential() {
        assert!((recency_boost(0.0, &RecencyModel::Exponential, 7.0) - 1.0).abs() < 1e-6);
        let b = recency_boost(7.0, &RecencyModel::Exponential, 7.0);
        assert!((b - 0.3679).abs() < 0.01, "Day 7 should be ~0.368, got {b}");
    }

    #[test]
    fn test_recency_boost_none() {
        assert!((recency_boost(0.0, &RecencyModel::None, 7.0) - 1.0).abs() < 1e-6);
        assert!((recency_boost(100.0, &RecencyModel::None, 7.0) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_recency_boost_zero_stability() {
        assert!((recency_boost(5.0, &RecencyModel::Fsrs, 0.0) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_cap_total_boost() {
        assert!((cap_total_boost(1.0, 1.0, 4.0) - 1.0).abs() < 1e-6);
        assert!((cap_total_boost(3.0, 2.0, 4.0) - 4.0).abs() < 1e-6);
        assert!((cap_total_boost(1.0, 1.0, 1.0) - 1.0).abs() < 1e-6);
    }
}
