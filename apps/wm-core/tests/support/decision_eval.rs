use std::collections::{BTreeMap, BTreeSet};
use std::time::Instant;

use serde::{Deserialize, Serialize};
use wm_engine::{
    canonical_questions, spec_for_question, AnswerValue, ClassificationSpec, DecisionAnswer,
    DecisionBackend, DecisionError, DecisionRecord, DecisionRuntime, PageType, Question, QType,
    RECORD_SCHEMA_VERSION,
};

pub const FIXTURE: &str = "tests/fixtures/decision_eval.jsonl";
pub const ENV_SERIALIZATION: &str = "WM_EVAL_SERIALIZATION";
pub const ENV_PHRASING: &str = "WM_EVAL_PHRASING";
pub const ENV_TRIM_WORDS: &str = "WM_EVAL_TRIM_WORDS";
pub const ENV_SCORE_DECODE: &str = "WM_EVAL_SCORE_DECODE";
pub const ENV_LIMIT: &str = "WM_EVAL_LIMIT";

const DEFAULT_TRIM_WORDS: usize = 200;
const FEW_SHOT_MAX: usize = 2;
const FEW_SHOT_WORDS: usize = 24;
const KEY_FACT_WORDS: usize = 24;
const TAGGED_PREFIX: &str = "[STATE]";
const HEADING_TAG: &str = "[HEADING]";
const HEADING_PREFIX: &str = "## ";
const KEY_FACT_PREFIX: &str = "- ";
const KEY_FACT_SEPARATOR: &str = ": ";
const SECTION_SEPARATOR: &str = "\n";
const DEFAULT_SECTION: &str = "body";
const SENTENCE_TERMINATOR: char = '.';
const NOUL_LABEL_FALSE: &str = "false";
const NOUL_LABEL_TRUE: &str = "true";
const NOUL_FALSE_DESCRIPTION: &str = "the statement does not hold for the state";
const NOUL_TRUE_DESCRIPTION: &str = "the statement holds for the state";
const CUMULATIVE_THRESHOLD: f64 = 0.5;
const ECE_BINS: usize = 5;
const PERCENT: f64 = 100.0;
const MILLIS_PER_SECOND: f64 = 1_000.0;
const F1_COEFFICIENT: f64 = 2.0;
const CHOICE_KEY: &str = "choice";
const SCORE_KEY: &str = "score";
const NOUL_KEY: &str = "noul";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Serialization {
    Prose,
    TaggedProse,
    Trimmed,
    CompactJson,
    KeyFacts,
}

impl Serialization {
    pub fn as_str(&self) -> &'static str {
        match self {
            Serialization::Prose => "prose",
            Serialization::TaggedProse => "tagged-prose",
            Serialization::Trimmed => "trimmed",
            Serialization::CompactJson => "compact-json",
            Serialization::KeyFacts => "key-facts",
        }
    }

    pub fn parse(value: &str) -> Option<Serialization> {
        match value {
            "prose" => Some(Serialization::Prose),
            "tagged-prose" => Some(Serialization::TaggedProse),
            "trimmed" => Some(Serialization::Trimmed),
            "compact-json" => Some(Serialization::CompactJson),
            "key-facts" => Some(Serialization::KeyFacts),
            _ => None,
        }
    }

    pub fn all() -> [Serialization; 5] {
        [
            Serialization::Prose,
            Serialization::TaggedProse,
            Serialization::Trimmed,
            Serialization::CompactJson,
            Serialization::KeyFacts,
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Phrasing {
    Raw,
    LabelDescriptions,
    FewShot,
}

impl Phrasing {
    pub fn as_str(&self) -> &'static str {
        match self {
            Phrasing::Raw => "raw",
            Phrasing::LabelDescriptions => "label-descriptions",
            Phrasing::FewShot => "few-shot",
        }
    }

    pub fn parse(value: &str) -> Option<Phrasing> {
        match value {
            "raw" => Some(Phrasing::Raw),
            "label-descriptions" => Some(Phrasing::LabelDescriptions),
            "few-shot" => Some(Phrasing::FewShot),
            _ => None,
        }
    }

    pub fn all() -> [Phrasing; 3] {
        [
            Phrasing::Raw,
            Phrasing::LabelDescriptions,
            Phrasing::FewShot,
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ScoreDecode {
    Argmax,
    Cumulative,
}

impl ScoreDecode {
    pub fn as_str(&self) -> &'static str {
        match self {
            ScoreDecode::Argmax => "argmax",
            ScoreDecode::Cumulative => "cumulative",
        }
    }

    pub fn parse(value: &str) -> Option<ScoreDecode> {
        match value {
            "argmax" => Some(ScoreDecode::Argmax),
            "cumulative" => Some(ScoreDecode::Cumulative),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct EvalConfig {
    pub serialization: Serialization,
    pub phrasing: Phrasing,
    pub score_decode: ScoreDecode,
    pub trim_words: usize,
}

impl Default for EvalConfig {
    fn default() -> Self {
        Self {
            serialization: Serialization::Prose,
            phrasing: Phrasing::Raw,
            score_decode: ScoreDecode::Argmax,
            trim_words: DEFAULT_TRIM_WORDS,
        }
    }
}

impl EvalConfig {
    pub fn new(
        serialization: Serialization,
        phrasing: Phrasing,
        score_decode: ScoreDecode,
        trim_words: usize,
    ) -> Self {
        Self {
            serialization,
            phrasing,
            score_decode,
            trim_words,
        }
    }

    pub fn default_trim_words() -> usize {
        DEFAULT_TRIM_WORDS
    }

    pub fn from_env() -> Self {
        let serialization = std::env::var(ENV_SERIALIZATION)
            .ok()
            .and_then(|value| Serialization::parse(&value))
            .unwrap_or(Serialization::Prose);
        let phrasing = std::env::var(ENV_PHRASING)
            .ok()
            .and_then(|value| Phrasing::parse(&value))
            .unwrap_or(Phrasing::Raw);
        let score_decode = std::env::var(ENV_SCORE_DECODE)
            .ok()
            .and_then(|value| ScoreDecode::parse(&value))
            .unwrap_or(ScoreDecode::Argmax);
        let trim_words = std::env::var(ENV_TRIM_WORDS)
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or(DEFAULT_TRIM_WORDS);
        Self::new(serialization, phrasing, score_decode, trim_words)
    }
}

pub fn env_row_limit() -> Option<usize> {
    std::env::var(ENV_LIMIT)
        .ok()
        .and_then(|value| value.parse().ok())
}

#[derive(Debug, Clone, Deserialize)]
pub struct EvalRow {
    pub page_type: PageType,
    pub state: String,
    #[serde(default)]
    pub state_json: Option<String>,
    pub expected: BTreeMap<String, serde_json::Value>,
    #[serde(default)]
    pub soft: BTreeSet<String>,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct PrimitiveStats {
    pub n: usize,
    pub correct: usize,
    pub accuracy: f64,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct EceBin {
    pub lower: f64,
    pub upper: f64,
    pub count: usize,
    pub accuracy: f64,
    pub mean_confidence: f64,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct EvalReport {
    pub rows: usize,
    pub evaluated_rows: usize,
    pub labeled: usize,
    pub config: EvalConfigView,
    pub overall: PrimitiveStats,
    pub by_primitive: BTreeMap<String, PrimitiveStats>,
    pub choice_macro_f1: Option<f64>,
    pub ece_noul: Option<f64>,
    pub ece_score: Option<f64>,
    pub ece_bins: BTreeMap<String, Vec<EceBin>>,
    pub brier: BTreeMap<String, f64>,
    pub latency_p50_ms: f64,
    pub latency_p90_ms: f64,
    pub baseline_accuracy: f64,
    pub state_words_mean: f64,
    pub prompt_words_mean: f64,
    pub soft_n: usize,
    pub soft_accuracy: f64,
    pub by_page_type: BTreeMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct EvalConfigView {
    pub serialization: String,
    pub phrasing: String,
    pub score_decode: String,
    pub trim_words: usize,
}

pub struct ClosureBackend<F> {
    classify: F,
}

impl<F> DecisionBackend for ClosureBackend<F>
where
    F: Fn(&str, &[ClassificationSpec]) -> Result<Vec<wm_engine::LabelProbabilities>, DecisionError>,
{
    fn classify(
        &self,
        text: &str,
        specs: &[ClassificationSpec],
    ) -> Result<Vec<wm_engine::LabelProbabilities>, DecisionError> {
        (self.classify)(text, specs)
    }
}

pub fn load_fixture() -> Vec<EvalRow> {
    let path = format!("{}/{FIXTURE}", env!("CARGO_MANIFEST_DIR"));
    let raw = std::fs::read_to_string(&path).expect("fixture must be readable");
    raw.lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str(line).expect("fixture row must parse"))
        .collect()
}

pub fn run_config<F>(
    classify: F,
    fixture: &[EvalRow],
    config: &EvalConfig,
) -> Result<EvalReport, DecisionError>
where
    F: Fn(&str, &[ClassificationSpec]) -> Result<Vec<wm_engine::LabelProbabilities>, DecisionError>,
{
    let runtime = DecisionRuntime::new(ClosureBackend { classify }, "eval");
    let mut observations: Vec<Observation> = Vec::new();
    let mut latencies: Vec<f64> = Vec::new();
    let mut evaluated: Vec<&EvalRow> = Vec::new();
    let mut state_words_total = 0usize;
    let mut prompt_words_total = 0usize;

    for (index, row) in fixture.iter().enumerate() {
        let Some(state) = serialize(row, config) else {
            continue;
        };
        let questions = canonical_questions(&row.page_type);
        let specs = build_specs(&row.page_type, &questions, row, fixture, index, config.phrasing);
        let record = DecisionRecord {
            schema_version: RECORD_SCHEMA_VERSION,
            state: state.clone(),
            questions: questions.clone(),
            answers: BTreeMap::new(),
        };

        let started = Instant::now();
        let result = runtime.answer_with_specs(None, &record, &specs)?;
        latencies.push(started.elapsed().as_secs_f64() * MILLIS_PER_SECOND);
        evaluated.push(row);
        state_words_total = state_words_total.saturating_add(state.split_whitespace().count());
        prompt_words_total = prompt_words_total.saturating_add(
            specs
                .iter()
                .map(|spec| spec.prompt.split_whitespace().count())
                .sum::<usize>(),
        );

        for question in &questions {
            let Some(expected) = row.expected.get(&question.id) else {
                continue;
            };
            let answer = result
                .answers
                .get(&question.id)
                .cloned()
                .ok_or_else(|| DecisionError::MissingTaskResult(question.id.clone()))?;
            let answer = apply_score_decode(question, answer, config.score_decode);
            observations.push(Observation {
                page_type: row.page_type.as_str().to_owned(),
                question_id: question.id.clone(),
                primitive: question.qtype,
                correct: value_matches(expected, &answer.value),
                confidence: confidence(question.qtype, &answer),
                distribution: answer.distribution.clone().unwrap_or_default(),
                expected_set: expected_set(expected),
                expected_label: expected_label(expected),
                predicted_label: predicted_label(&answer.value),
                soft: row.soft.contains(&question.id),
            });
        }
    }

    Ok(report(
        &observations,
        &latencies,
        &evaluated,
        config,
        mean_of(state_words_total, evaluated.len()),
        mean_of(prompt_words_total, evaluated.len()),
    ))
}

fn serialize(row: &EvalRow, config: &EvalConfig) -> Option<String> {
    match config.serialization {
        Serialization::Prose => Some(row.state.clone()),
        Serialization::TaggedProse => Some(format!("{TAGGED_PREFIX}\n{}", tag_headings(&row.state))),
        Serialization::Trimmed => Some(first_words(&row.state, config.trim_words)),
        Serialization::CompactJson => row.state_json.clone(),
        Serialization::KeyFacts => Some(key_facts(&row.state)),
    }
}

fn tag_headings(state: &str) -> String {
    state
        .lines()
        .map(|line| match line.strip_prefix(HEADING_PREFIX) {
            Some(title) => format!("{HEADING_TAG} {title}"),
            None => line.to_owned(),
        })
        .collect::<Vec<_>>()
        .join(SECTION_SEPARATOR)
}

fn key_facts(state: &str) -> String {
    sections(state)
        .iter()
        .map(|(heading, body)| {
            format!(
                "{KEY_FACT_PREFIX}{heading}{KEY_FACT_SEPARATOR}{}",
                first_sentence(body)
            )
        })
        .collect::<Vec<_>>()
        .join(SECTION_SEPARATOR)
}

fn sections(state: &str) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = Vec::new();
    let mut heading = DEFAULT_SECTION.to_owned();
    let mut body: Vec<&str> = Vec::new();
    for line in state.lines() {
        match line.strip_prefix(HEADING_PREFIX) {
            Some(title) => {
                flush_section(&mut out, &heading, &body);
                heading = title.trim().to_owned();
                body.clear();
            }
            None => body.push(line),
        }
    }
    flush_section(&mut out, &heading, &body);
    out
}

fn flush_section(out: &mut Vec<(String, String)>, heading: &str, body: &[&str]) {
    let body = body.join(SECTION_SEPARATOR);
    if body.trim().is_empty() {
        return;
    }
    out.push((heading.to_owned(), body));
}

fn first_sentence(body: &str) -> String {
    let trimmed = body.trim();
    match trimmed.split_once(SENTENCE_TERMINATOR) {
        Some((sentence, _)) if !sentence.trim().is_empty() => sentence.trim().to_owned(),
        _ => first_words(trimmed, KEY_FACT_WORDS),
    }
}

fn build_specs(
    page_type: &PageType,
    questions: &[Question],
    row: &EvalRow,
    fixture: &[EvalRow],
    index: usize,
    phrasing: Phrasing,
) -> Vec<ClassificationSpec> {
    questions
        .iter()
        .map(|question| {
            let mut spec = spec_for_question(question);
            match phrasing {
                Phrasing::Raw => {}
                Phrasing::LabelDescriptions => {
                    spec.label_descriptions = label_descriptions_for(page_type, question);
                }
                Phrasing::FewShot => {
                    spec.examples = few_shot_examples(&row.page_type, question, fixture, index);
                }
            }
            spec
        })
        .collect()
}

fn label_descriptions_for(page_type: &PageType, question: &Question) -> Vec<(String, String)> {
    if question.qtype == QType::Noul {
        return vec![
            (
                NOUL_LABEL_FALSE.to_owned(),
                NOUL_FALSE_DESCRIPTION.to_owned(),
            ),
            (NOUL_LABEL_TRUE.to_owned(), NOUL_TRUE_DESCRIPTION.to_owned()),
        ];
    }
    question_labels(question)
        .iter()
        .filter_map(|label| {
            label_description(page_type.as_str(), &question.id, label)
                .map(|description| ((*label).to_owned(), description.to_owned()))
        })
        .collect()
}

fn few_shot_examples(
    page_type: &PageType,
    question: &Question,
    fixture: &[EvalRow],
    index: usize,
) -> Vec<(String, String)> {
    let mut examples: Vec<(String, String)> = Vec::new();
    collect_examples(fixture, index, Some(page_type), question, &mut examples);
    if examples.is_empty() {
        collect_examples(fixture, index, None, question, &mut examples);
    }
    examples.truncate(FEW_SHOT_MAX);
    examples
}

fn collect_examples(
    fixture: &[EvalRow],
    index: usize,
    page_type: Option<&PageType>,
    question: &Question,
    out: &mut Vec<(String, String)>,
) {
    for (other_index, other) in fixture.iter().enumerate() {
        if other_index == index || out.len() >= FEW_SHOT_MAX {
            continue;
        }
        if other.soft.contains(&question.id) {
            continue;
        }
        if let Some(page_type) = page_type {
            if &other.page_type != page_type {
                continue;
            }
        }
        let Some(value) = other.expected.get(&question.id) else {
            continue;
        };
        let snippet = first_words(&other.state, FEW_SHOT_WORDS);
        for label in expected_labels(value) {
            if out.len() >= FEW_SHOT_MAX {
                break;
            }
            out.push((snippet.clone(), label));
        }
    }
}

fn question_labels(question: &Question) -> Vec<&str> {
    match question.qtype {
        QType::Choice => question.options.iter().map(String::as_str).collect(),
        QType::Score => question.levels.iter().map(String::as_str).collect(),
        QType::Noul => Vec::new(),
    }
}

fn first_words(text: &str, count: usize) -> String {
    text.split_whitespace().take(count).collect::<Vec<_>>().join(" ")
}

fn apply_score_decode(
    question: &Question,
    answer: DecisionAnswer,
    decode: ScoreDecode,
) -> DecisionAnswer {
    if decode != ScoreDecode::Cumulative || question.qtype != QType::Score {
        return answer;
    }
    let levels = &question.levels;
    if levels.is_empty() {
        return answer;
    }
    let mut index = 0usize;
    for k in 1..levels.len() {
        let cumulative: f64 = levels[k..]
            .iter()
            .filter_map(|level| answer.distribution.as_ref()?.get(level).copied())
            .sum();
        if cumulative >= CUMULATIVE_THRESHOLD {
            index = k;
        }
    }
    let probability = answer
        .distribution
        .as_ref()
        .and_then(|distribution| distribution.get(&levels[index]))
        .copied()
        .unwrap_or_default();
    DecisionAnswer {
        value: AnswerValue::Label(levels[index].clone()),
        probability,
        distribution: answer.distribution,
    }
}

#[derive(Clone)]
struct Observation {
    page_type: String,
    question_id: String,
    primitive: QType,
    correct: bool,
    confidence: Option<f64>,
    distribution: BTreeMap<String, f64>,
    expected_set: BTreeSet<String>,
    expected_label: String,
    predicted_label: String,
    soft: bool,
}

fn report(
    observations: &[Observation],
    latencies: &[f64],
    evaluated: &[&EvalRow],
    config: &EvalConfig,
    state_words_mean: f64,
    prompt_words_mean: f64,
) -> EvalReport {
    let overall = stats(observations);
    let mut by_primitive: BTreeMap<String, PrimitiveStats> = BTreeMap::new();
    for primitive in [QType::Choice, QType::Score, QType::Noul] {
        by_primitive.insert(
            primitive.as_str().to_owned(),
            primitive_stats(observations, primitive),
        );
    }

    let mut by_page_type: BTreeMap<String, f64> = BTreeMap::new();
    let page_types: BTreeSet<&str> = observations
        .iter()
        .map(|observation| observation.page_type.as_str())
        .collect();
    for page_type in page_types {
        let subset: Vec<Observation> = observations
            .iter()
            .filter(|observation| observation.page_type == page_type)
            .cloned()
            .collect();
        by_page_type.insert(page_type.to_owned(), stats(&subset).accuracy);
    }

    let soft: Vec<Observation> = observations
        .iter()
        .filter(|observation| observation.soft)
        .cloned()
        .collect();
    let noul = ece(observations, QType::Noul);
    let score = ece(observations, QType::Score);
    let mut ece_bins: BTreeMap<String, Vec<EceBin>> = BTreeMap::new();
    ece_bins.insert(
        NOUL_KEY.to_owned(),
        noul.as_ref().map(|result| result.bins.clone()).unwrap_or_default(),
    );
    ece_bins.insert(
        SCORE_KEY.to_owned(),
        score
            .as_ref()
            .map(|result| result.bins.clone())
            .unwrap_or_default(),
    );
    let brier: BTreeMap<String, f64> = [
        (CHOICE_KEY, brier(observations, QType::Choice)),
        (SCORE_KEY, brier(observations, QType::Score)),
        (NOUL_KEY, brier(observations, QType::Noul)),
    ]
    .into_iter()
    .filter_map(|(key, value)| value.map(|value| (key.to_owned(), value)))
    .collect();

    EvalReport {
        rows: evaluated.len(),
        evaluated_rows: evaluated.len(),
        labeled: observations.len(),
        config: EvalConfigView {
            serialization: config.serialization.as_str().to_owned(),
            phrasing: config.phrasing.as_str().to_owned(),
            score_decode: config.score_decode.as_str().to_owned(),
            trim_words: config.trim_words,
        },
        overall,
        by_primitive,
        choice_macro_f1: macro_f1(observations),
        ece_noul: noul.map(|result| result.ece),
        ece_score: score.map(|result| result.ece),
        ece_bins,
        brier,
        latency_p50_ms: percentile(latencies, 50.0),
        latency_p90_ms: percentile(latencies, 90.0),
        baseline_accuracy: baseline_accuracy(evaluated),
        state_words_mean,
        prompt_words_mean,
        soft_n: soft.len(),
        soft_accuracy: stats(&soft).accuracy,
        by_page_type,
    }
}

fn primitive_stats(observations: &[Observation], primitive: QType) -> PrimitiveStats {
    let subset: Vec<Observation> = observations
        .iter()
        .filter(|observation| observation.primitive == primitive)
        .cloned()
        .collect();
    stats(&subset)
}

fn stats(observations: &[Observation]) -> PrimitiveStats {
    let n = observations.len();
    let correct = observations
        .iter()
        .filter(|observation| observation.correct)
        .count();
    PrimitiveStats {
        n,
        correct,
        accuracy: ratio(correct, n),
    }
}

fn mean_of(total: usize, count: usize) -> f64 {
    match count {
        0 => 0.0,
        _ => total as f64 / count as f64,
    }
}

fn ratio(numerator: usize, denominator: usize) -> f64 {
    match denominator {
        0 => 0.0,
        _ => numerator as f64 / denominator as f64,
    }
}

fn macro_f1(observations: &[Observation]) -> Option<f64> {
    let choice: Vec<&Observation> = observations
        .iter()
        .filter(|observation| observation.primitive == QType::Choice)
        .collect();
    if choice.is_empty() {
        return None;
    }
    let groups: BTreeSet<(String, String)> = choice
        .iter()
        .map(|observation| {
            (
                observation.page_type.clone(),
                observation.question_id.clone(),
            )
        })
        .collect();
    let mut scores: Vec<f64> = Vec::new();
    for group in groups {
        let subset: Vec<&Observation> = choice
            .iter()
            .copied()
            .filter(|observation| {
                observation.page_type == group.0 && observation.question_id == group.1
            })
            .collect();
        scores.push(group_macro_f1(&subset));
    }
    Some(mean(&scores))
}

fn group_macro_f1(observations: &[&Observation]) -> f64 {
    let classes: BTreeSet<&str> = observations
        .iter()
        .flat_map(|observation| {
            [
                observation.expected_label.as_str(),
                observation.predicted_label.as_str(),
            ]
        })
        .collect();
    let mut scores: Vec<f64> = Vec::new();
    for class in classes {
        let tp = observations
            .iter()
            .filter(|observation| {
                observation.expected_label == class && observation.predicted_label == class
            })
            .count();
        let fp = observations
            .iter()
            .filter(|observation| {
                observation.expected_label != class && observation.predicted_label == class
            })
            .count();
        let fnn = observations
            .iter()
            .filter(|observation| {
                observation.expected_label == class && observation.predicted_label != class
            })
            .count();
        scores.push(f1(tp, fp, fnn));
    }
    mean(&scores)
}

fn f1(true_positive: usize, false_positive: usize, false_negative: usize) -> f64 {
    let precision = ratio(true_positive, true_positive + false_positive);
    let recall = ratio(true_positive, true_positive + false_negative);
    if precision + recall == 0.0 {
        return 0.0;
    }
    F1_COEFFICIENT * precision * recall / (precision + recall)
}

#[derive(Clone)]
struct EceResult {
    ece: f64,
    bins: Vec<EceBin>,
}

fn ece(observations: &[Observation], primitive: QType) -> Option<EceResult> {
    let subset: Vec<&Observation> = observations
        .iter()
        .filter(|observation| observation.primitive == primitive)
        .filter(|observation| observation.confidence.is_some())
        .collect();
    if subset.is_empty() {
        return None;
    }
    let total = subset.len();
    let width = 1.0 / ECE_BINS as f64;
    let mut counts = [0usize; ECE_BINS];
    let mut correct = [0usize; ECE_BINS];
    let mut confidence_sum = [0.0_f64; ECE_BINS];
    for observation in &subset {
        let confidence = observation.confidence.unwrap_or_default().clamp(0.0, 1.0);
        let index = ((confidence / width).floor() as usize).min(ECE_BINS - 1);
        counts[index] += 1;
        correct[index] += usize::from(observation.correct);
        confidence_sum[index] += confidence;
    }
    let bins: Vec<EceBin> = (0..ECE_BINS)
        .map(|index| {
            let lower = index as f64 * width;
            let count = counts[index];
            EceBin {
                lower,
                upper: lower + width,
                count,
                accuracy: ratio(correct[index], count),
                mean_confidence: ratio_f64(confidence_sum[index], count),
            }
        })
        .collect();
    let ece = bins
        .iter()
        .map(|bin| (bin.count as f64 / total as f64) * (bin.accuracy - bin.mean_confidence).abs())
        .sum();
    Some(EceResult { ece, bins })
}

fn brier(observations: &[Observation], primitive: QType) -> Option<f64> {
    let subset: Vec<&Observation> = observations
        .iter()
        .filter(|observation| observation.primitive == primitive)
        .filter(|observation| !observation.distribution.is_empty())
        .collect();
    if subset.is_empty() {
        return None;
    }
    let scores: Vec<f64> = subset
        .iter()
        .map(|observation| {
            observation
                .distribution
                .iter()
                .map(|(label, probability)| {
                    let actual = f64::from(observation.expected_set.contains(label));
                    let difference = probability - actual;
                    difference * difference
                })
                .sum()
        })
        .collect();
    Some(mean(&scores))
}

fn baseline_accuracy(evaluated: &[&EvalRow]) -> f64 {
    let mut referenced: BTreeMap<(String, String), Vec<&EvalRow>> = BTreeMap::new();
    for row in evaluated {
        let page_type = row.page_type.as_str().to_owned();
        for question_id in row.expected.keys() {
            referenced
                .entry((page_type.clone(), question_id.clone()))
                .or_default()
                .push(row);
        }
    }

    let mut correct = 0usize;
    let mut total = 0usize;
    for ((_page_type, question_id), rows) in referenced {
        let mut counts: BTreeMap<String, usize> = BTreeMap::new();
        for row in &rows {
            let key = row
                .expected
                .get(&question_id)
                .map(expected_label)
                .unwrap_or_default();
            *counts.entry(key).or_insert(0) += 1;
        }
        let constant = counts
            .iter()
            .max_by(|left, right| left.1.cmp(right.1).then_with(|| left.0.cmp(right.0)))
            .map(|(label, _)| label.clone())
            .unwrap_or_default();
        for row in &rows {
            total += 1;
            let key = row
                .expected
                .get(&question_id)
                .map(expected_label)
                .unwrap_or_default();
            correct += usize::from(key == constant);
        }
    }
    ratio(correct, total)
}

fn percentile(values: &[f64], percentile: f64) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(|left, right| left.partial_cmp(right).unwrap_or(std::cmp::Ordering::Equal));
    let rank = (percentile / PERCENT * sorted.len() as f64).ceil() as usize;
    let index = rank.saturating_sub(1).min(sorted.len() - 1);
    sorted[index]
}

fn mean(values: &[f64]) -> f64 {
    match values.is_empty() {
        true => 0.0,
        false => values.iter().sum::<f64>() / values.len() as f64,
    }
}

fn ratio_f64(numerator: f64, denominator: usize) -> f64 {
    match denominator {
        0 => 0.0,
        _ => numerator / denominator as f64,
    }
}

fn confidence(qtype: QType, answer: &DecisionAnswer) -> Option<f64> {
    match qtype {
        QType::Noul => match answer.value {
            AnswerValue::Bool(true) => Some(answer.probability),
            AnswerValue::Bool(false) => Some(1.0 - answer.probability),
            _ => None,
        },
        QType::Score => Some(answer.probability),
        QType::Choice => None,
    }
}

fn value_matches(expected: &serde_json::Value, predicted: &AnswerValue) -> bool {
    match (expected, predicted) {
        (serde_json::Value::Bool(expected), AnswerValue::Bool(predicted)) => expected == predicted,
        (serde_json::Value::String(expected), AnswerValue::Label(predicted)) => {
            expected == predicted
        }
        (serde_json::Value::Array(expected), AnswerValue::Labels(predicted)) => {
            let mut expected: Vec<&str> = expected.iter().filter_map(|v| v.as_str()).collect();
            let mut predicted: Vec<&str> = predicted.iter().map(|s| s.as_str()).collect();
            expected.sort_unstable();
            predicted.sort_unstable();
            expected == predicted
        }
        _ => false,
    }
}

fn expected_label(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Bool(flag) => flag.to_string(),
        serde_json::Value::String(label) => label.clone(),
        serde_json::Value::Array(labels) => {
            let mut labels: Vec<&str> = labels.iter().filter_map(|v| v.as_str()).collect();
            labels.sort_unstable();
            labels.join("|")
        }
        other => other.to_string(),
    }
}

fn expected_labels(value: &serde_json::Value) -> Vec<String> {
    match value {
        serde_json::Value::Bool(flag) => vec![flag.to_string()],
        serde_json::Value::String(label) => vec![label.clone()],
        serde_json::Value::Array(labels) => labels
            .iter()
            .filter_map(|value| value.as_str().map(str::to_owned))
            .collect(),
        other => vec![other.to_string()],
    }
}

fn expected_set(value: &serde_json::Value) -> BTreeSet<String> {
    expected_labels(value).into_iter().collect()
}

fn predicted_label(value: &AnswerValue) -> String {
    match value {
        AnswerValue::Bool(flag) => flag.to_string(),
        AnswerValue::Label(label) => label.clone(),
        AnswerValue::Labels(labels) => {
            let mut labels: Vec<&str> = labels.iter().map(|s| s.as_str()).collect();
            labels.sort_unstable();
            labels.join("|")
        }
    }
}

fn label_description(page_type: &str, question_id: &str, label: &str) -> Option<&'static str> {
    let description = match (page_type, question_id, label) {
        ("decision", "outcome", "adopted") => "the option was chosen and is now in force",
        ("decision", "outcome", "rejected") => "the option was considered but not chosen",
        ("decision", "outcome", "deferred") => "a choice was postponed to a later decision",
        ("decision", "outcome", "superseded") => "a newer decision replaced this one",
        ("decision", "outcome", "abandoned") => "the option was dropped without a replacement",
        ("decision", "impact", "local") => "confined to a single module or function",
        ("decision", "impact", "component") => "affects one component or subsystem",
        ("decision", "impact", "system") => "affects several interacting subsystems",
        ("decision", "impact", "project-wide") => "changes cross-cutting project behaviour",
        ("decision", "confidence", "low") => "little or no supporting rationale",
        ("decision", "confidence", "medium") => "some rationale but gaps remain",
        ("decision", "confidence", "high") => "strong, documented rationale",
        ("pattern", "problem_kind", "architecture") => "structural or concurrency design",
        ("pattern", "problem_kind", "api-design") => "shapes a public interface or signature",
        ("pattern", "problem_kind", "data-model") => "changes stored data representation",
        ("pattern", "problem_kind", "error-handling") => "changes how failures surface",
        ("pattern", "problem_kind", "performance") => "reduces time or space cost",
        ("pattern", "problem_kind", "testing") => "changes test structure or strategy",
        ("pattern", "problem_kind", "ui") => "user- or console-visible output",
        ("pattern", "problem_kind", "tooling") => "developer tooling or scripts",
        ("pattern", "problem_kind", "workflow") => "a team or agent process step",
        ("pattern", "complexity", "trivial") => "no design decisions, applied mechanically",
        ("pattern", "complexity", "simple") => "one small step with obvious choices",
        ("pattern", "complexity", "moderate") => "several steps with trade-offs",
        ("pattern", "complexity", "complex") => "many interacting parts and trade-offs",
        ("concept", "kind", "concept") => "explains a domain idea or mechanism",
        ("concept", "kind", "failure-analysis") => "diagnoses a defect or incident",
        ("concept", "kind", "research-report") => "reports findings of an investigation",
        ("concept", "kind", "reference-note") => "a short lookup note, not an explanation",
        ("concept", "category", "architecture") => "system structure or engine internals",
        ("concept", "category", "search-retrieval") => "search, ranking or retrieval",
        ("concept", "category", "graph") => "wiki or code graph structure",
        ("concept", "category", "parser-format") => "parsing, markdown or file formats",
        ("concept", "category", "mcp-tooling") => "the MCP tool surface",
        ("concept", "category", "cli") => "command-line user experience",
        ("concept", "category", "storage") => "databases or persistence",
        ("concept", "category", "embeddings") => "vectors or embedding models",
        ("concept", "category", "web-ui") => "the browser user interface",
        ("concept", "category", "process") => "agent or team process",
        ("concept", "maturity", "raw") => "initial, unverified notes",
        ("concept", "maturity", "exploratory") => "idea under active exploration",
        ("concept", "maturity", "established") => "well understood and used in practice",
        ("concept", "maturity", "stable") => "settled and unlikely to change",
        ("howto", "task_kind", "setup") => "installing or configuring an environment",
        ("howto", "task_kind", "development") => "writing or changing code",
        ("howto", "task_kind", "testing") => "running or writing tests",
        ("howto", "task_kind", "release") => "cutting or publishing a release",
        ("howto", "task_kind", "debugging") => "diagnosing a problem",
        ("howto", "task_kind", "operations") => "operating a running system",
        ("howto", "task_kind", "integration") => "connecting an external service",
        ("howto", "difficulty", "beginner") => "no prior context needed",
        ("howto", "difficulty", "intermediate") => "some familiarity with the area",
        ("howto", "difficulty", "advanced") => "requires deep area knowledge",
        ("howto", "difficulty", "expert") => "requires specialist expertise",
        ("reference", "kind", "api") => "callable entry points such as functions or tools",
        ("reference", "kind", "cli") => "shell commands and their flags",
        ("reference", "kind", "configuration") => "settings or fields a user sets",
        ("reference", "kind", "error-catalog") => "error codes and their causes",
        ("reference", "kind", "schema") => "the shape and fields of a data structure",
        ("reference", "kind", "scoring") => "ranking formulas or weights",
        ("reference", "surface", "mcp-tool") => "an MCP tool call",
        ("reference", "surface", "cli-command") => "a CLI invocation",
        ("reference", "surface", "rust-api") => "a Rust library API",
        ("reference", "surface", "http-api") => "an HTTP endpoint",
        ("reference", "surface", "config-file") => "a configuration file",
        ("reference", "stability", "unstable") => "may change without notice",
        ("reference", "stability", "evolving") => "changing, breaking changes possible",
        ("reference", "stability", "stable") => "rarely changes",
        ("reference", "stability", "frozen") => "guaranteed not to change",
        _ => return None,
    };
    Some(description)
}

pub fn render_report(report: &EvalReport) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "config={} rows={} evaluated={} labeled={} overall_acc={:.3} baseline_acc={:.3} soft_n={} soft_acc={:.3}\n",
        render_config(report),
        report.rows,
        report.evaluated_rows,
        report.labeled,
        report.overall.accuracy,
        report.baseline_accuracy,
        report.soft_n,
        report.soft_accuracy
    ));
    for (primitive, stats) in &report.by_primitive {
        out.push_str(&format!(
            "  {primitive:>6}: n={:>3} acc={:.3} ({}/{})\n",
            stats.n, stats.accuracy, stats.correct, stats.n
        ));
    }
    out.push_str(&format!(
        "  macro_f1(choice)={:?} ece_noul={:?} ece_score={:?}\n",
        report.choice_macro_f1, report.ece_noul, report.ece_score
    ));
    out.push_str(&format!(
        "  brier={:?} state_words_mean={:.1} prompt_words_mean={:.1}\n",
        report.brier, report.state_words_mean, report.prompt_words_mean
    ));
    out.push_str(&format!(
        "  latency p50={:.1}ms p90={:.1}ms\n",
        report.latency_p50_ms, report.latency_p90_ms
    ));
    for (page_type, accuracy) in &report.by_page_type {
        out.push_str(&format!("  {page_type:>9}: acc={accuracy:.3}\n"));
    }
    out
}

fn render_config(report: &EvalReport) -> String {
    format!(
        "serialization={} phrasing={} score_decode={} trim_words={}",
        report.config.serialization,
        report.config.phrasing,
        report.config.score_decode,
        report.config.trim_words
    )
}

pub fn render_config_line(model: &str, report: &EvalReport) -> String {
    let primitive_acc = |key: &str| {
        report
            .by_primitive
            .get(key)
            .map(|stats| stats.accuracy)
            .unwrap_or_default()
    };
    let brier = |key: &str| report.brier.get(key).copied().unwrap_or_default();
    format!(
        "config model={model} serialization={} phrasing={} score_decode={} trim_words={} rows={} evaluated={} labeled={} acc={:.3} baseline={:.3} choice_acc={:.3} macro_f1={:?} noul_acc={:.3} ece_noul={:?} score_acc={:.3} ece_score={:?} brier_choice={:.3} brier_noul={:.3} brier_score={:.3} state_words_mean={:.1} prompt_words_mean={:.1} p50_ms={:.1} p90_ms={:.1}",
        report.config.serialization,
        report.config.phrasing,
        report.config.score_decode,
        report.config.trim_words,
        report.rows,
        report.evaluated_rows,
        report.labeled,
        report.overall.accuracy,
        report.baseline_accuracy,
        primitive_acc(CHOICE_KEY),
        report.choice_macro_f1,
        primitive_acc(NOUL_KEY),
        report.ece_noul,
        primitive_acc(SCORE_KEY),
        report.ece_score,
        brier(CHOICE_KEY),
        brier(NOUL_KEY),
        brier(SCORE_KEY),
        report.state_words_mean,
        report.prompt_words_mean,
        report.latency_p50_ms,
        report.latency_p90_ms
    )
}
