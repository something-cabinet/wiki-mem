#[path = "support/decision_eval.rs"]
mod eval;

use eval::{EvalConfig, Phrasing, ScoreDecode, Serialization};
use wm_engine::{ClassificationSpec, DecisionError, LabelProbabilities};

fn constant_backend(
    probability: f32,
) -> impl Fn(&str, &[ClassificationSpec]) -> Result<Vec<LabelProbabilities>, DecisionError> {
    move |_text, specs| {
        Ok(specs
            .iter()
            .map(|spec| LabelProbabilities {
                task: spec.task.clone(),
                labels: spec.labels.clone(),
                probabilities: vec![probability; spec.labels.len()],
            })
            .collect())
    }
}

#[test]
fn mock_harness_computes_metrics() {
    let fixture = eval::load_fixture();
    assert!(fixture.len() >= 30, "fixture should carry ~30 rows");
    let labeled: usize = fixture.iter().map(|row| row.expected.len()).sum();
    assert_eq!(labeled, 114, "fixture should carry 114 labeled questions");

    let report = eval::run_config(constant_backend(0.0), &fixture, &EvalConfig::default())
        .expect("mock run should succeed");
    assert_eq!(report.rows, fixture.len());
    assert_eq!(report.evaluated_rows, fixture.len());
    assert_eq!(report.labeled, labeled);
    assert!((0.0..=1.0).contains(&report.overall.accuracy));
    assert!(report.latency_p50_ms >= 0.0);
    assert!(report.choice_macro_f1.is_some());
    assert!(report.ece_noul.is_some());
    assert!(report.ece_score.is_some());
    assert!((0.0..=1.0).contains(&report.baseline_accuracy));
    assert!(report.soft_n >= 20);
    assert!((0.0..=1.0).contains(&report.soft_accuracy));
    assert_eq!(report.ece_bins["noul"].len(), 5);
    assert!(report.brier.contains_key("score"));
    assert!(report.state_words_mean > 0.0);
    assert!(report.prompt_words_mean > 0.0);

    let _ = eval::env_row_limit();
    let from_env = EvalConfig::from_env();
    assert!(EvalConfig::default_trim_words() >= 1);
    assert!(!from_env.serialization.as_str().is_empty());

    println!("{}", eval::render_report(&report));
}

#[test]
fn mock_harness_runs_every_serialization_and_phrasing_mode() {
    let fixture = eval::load_fixture();
    assert!(
        fixture.iter().all(|row| row.state_json.is_some()),
        "every fixture row must carry state_json"
    );

    for serialization in Serialization::all() {
        for phrasing in Phrasing::all() {
            let config = EvalConfig::new(serialization, phrasing, ScoreDecode::Argmax, 120);
            let report = eval::run_config(constant_backend(0.0), &fixture, &config)
                .expect("mock mode run should succeed");
            assert_eq!(
                report.evaluated_rows,
                fixture.len(),
                "evaluated rows for {}",
                serialization.as_str()
            );
            assert!(report.choice_macro_f1.is_some());
            assert!(report.ece_noul.is_some());
            println!("{}", eval::render_config_line("mock", &report));
        }
    }
}

#[test]
fn mock_harness_supports_cumulative_score_decode() {
    let fixture = eval::load_fixture();
    let config = EvalConfig::new(
        Serialization::Prose,
        Phrasing::Raw,
        ScoreDecode::Cumulative,
        200,
    );
    let report = eval::run_config(constant_backend(0.0), &fixture, &config)
        .expect("cumulative mock run should succeed");
    assert_eq!(report.evaluated_rows, fixture.len());
    assert_eq!(report.config.score_decode, "cumulative");
    assert!(report.brier.contains_key("score"));
}
