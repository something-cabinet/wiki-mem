//! Golden-query evaluation harness for the live search pipeline.
//!
//! Measures macro-averaged recall@5 of `wm_search.query` (keyword mode)
//! against a fixed corpus of representative wiki pages with hand-labelled
//! expected results. The harness is the measurement instrument used to guard
//! ranking-tier changes: run it before and after a ranking change and compare
//! the printed `GOLDEN_EVAL recall@5` line.
//!
//! It is `#[ignore]`d so it never blocks the default CI suite (the ranking
//! evaluation is advisory, not a hard gate). Run it explicitly:
//!
//! ```bash
//! cargo test --no-default-features --features "code-intel,lsp" \
//!     -p wm-core --test golden_eval -- --ignored --nocapture
//! ```
//!
//! recall@5 for a single query is `|expected ∩ top5| / |expected|`; the
//! harness reports the mean across all golden queries. Keyword mode is used
//! because it exercises the BM25 field-weight + rerank tiers deterministically
//! without requiring an ONNX model to be present.

#[path = "helpers/inproc.rs"]
mod inproc;
use inproc::{call_ok, setup_in_process};

use serde_json::json;
use std::path::Path;

/// Number of top-ranked results counted as a hit for recall@k.
const RECALL_K: usize = 5;

/// Results requested per query. Larger than `RECALL_K` so the harness observes
/// ranking beyond the cut-off without changing the recall definition.
const RETRIEVE_LIMIT: usize = 10;

/// Minimum acceptable macro-averaged recall@5. Set conservatively below the
/// measured baseline so ordinary ranking tweaks do not flake the harness while
/// a genuine regression (a query's expected page dropping out of the top 5)
/// still trips it.
const RECALL_FLOOR: f64 = 0.85;

struct CorpusPage {
    path: &'static str,
    title: &'static str,
    tags: &'static str,
    body: &'static str,
}

struct GoldenQuery {
    query: &'static str,
    expected: &'static [&'static str],
}

const CORPUS: &[CorpusPage] = &[
    CorpusPage {
        path: "concepts/bm25-scoring",
        title: "BM25 Scoring",
        tags: "search, ranking",
        body: "BM25 ranks documents using term frequency and inverse document frequency. \
The k1 parameter controls term-frequency saturation and b controls length normalization. \
Each field contributes a weighted score to the final relevance total.",
    },
    CorpusPage {
        path: "concepts/rrf-fusion",
        title: "Reciprocal Rank Fusion",
        tags: "search, ranking",
        body: "Reciprocal rank fusion combines several ranked result lists into one. \
Each document earns one over k plus rank from every list, then the summed scores decide the fused order.",
    },
    CorpusPage {
        path: "concepts/cosine-similarity",
        title: "Cosine Similarity",
        tags: "vectors, math",
        body: "Cosine similarity measures the angle between two vectors as a normalized dot product. \
It divides the dot product by the product of the vector magnitudes so length does not affect the score.",
    },
    CorpusPage {
        path: "concepts/onnx-embeddings",
        title: "ONNX Embeddings",
        tags: "embeddings, ml",
        body: "ONNX runs a transformer model to produce neural sentence embeddings. \
The model turns text into dense vectors suitable for semantic comparison during inference.",
    },
    CorpusPage {
        path: "concepts/inverted-index",
        title: "Inverted Index",
        tags: "search, storage",
        body: "An inverted index maps each term to a postings list of the documents that contain it. \
Query evaluation intersects postings lists to find matching documents quickly.",
    },
    CorpusPage {
        path: "concepts/snowball-stemming",
        title: "Snowball Stemming",
        tags: "search, nlp",
        body: "Snowball applies the Porter2 algorithm to reduce words to their morphological stem. \
Stemming maps running and runs to run so query and document forms match.",
    },
    CorpusPage {
        path: "concepts/tokenization",
        title: "Code Aware Tokenizer",
        tags: "search, nlp",
        body: "A code aware tokenizer splits identifiers on snake case and camel case boundaries. \
It emits sub tokens so getUserName also matches user and name.",
    },
    CorpusPage {
        path: "concepts/knowledge-graph",
        title: "Knowledge Graph",
        tags: "graph, data",
        body: "A knowledge graph stores entities as nodes connected by typed edges. \
Traversal over the typed edges answers relationship questions between nodes.",
    },
    CorpusPage {
        path: "concepts/graph-centrality",
        title: "Graph Centrality",
        tags: "graph, ranking",
        body: "Graph centrality scores a node by the weighted sum of its inbound edges. \
Nodes referenced by many important edges receive a higher centrality.",
    },
    CorpusPage {
        path: "patterns/rate-limiting",
        title: "Rate Limiting",
        tags: "resilience, api",
        body: "Rate limiting throttles requests using a token bucket or leaky bucket. \
The token bucket refills at a fixed rate and rejects requests when empty.",
    },
    CorpusPage {
        path: "patterns/circuit-breaker",
        title: "Circuit Breaker",
        tags: "resilience",
        body: "A circuit breaker fails fast when a downstream dependency is unhealthy. \
It moves between closed, open, and half open states to probe recovery.",
    },
    CorpusPage {
        path: "patterns/retry-backoff",
        title: "Exponential Backoff Retry",
        tags: "resilience",
        body: "Exponential backoff retries transient failures with growing delays plus random jitter. \
Jitter spreads retries so clients do not synchronize into a thundering herd.",
    },
    CorpusPage {
        path: "patterns/cache-eviction",
        title: "Cache Eviction LRU",
        tags: "cache, performance",
        body: "An LRU cache evicts the least recently used entry when it reaches capacity. \
The eviction policy keeps hot keys and discards cold keys to bound memory.",
    },
    CorpusPage {
        path: "patterns/cache-aside",
        title: "Cache Aside",
        tags: "cache, performance",
        body: "Cache aside lazily loads data into the cache on a miss and reads through afterwards. \
The application populates the cache after fetching from the backing store.",
    },
    CorpusPage {
        path: "reference/redis-config",
        title: "Redis Configuration",
        tags: "redis, ops",
        body: "Redis configuration sets maxmemory and an eviction policy for persistence. \
It supports RDB snapshots and AOF append only logging for durability.",
    },
    CorpusPage {
        path: "reference/postgres-index",
        title: "Postgres Indexing",
        tags: "postgres, sql",
        body: "Postgres supports btree, gin, and gist index types for query planning. \
The planner chooses an index to avoid a sequential scan on large tables.",
    },
    CorpusPage {
        path: "reference/http-status-codes",
        title: "HTTP Status Codes",
        tags: "http, api",
        body: "HTTP status codes signal outcomes such as 200 success, 404 not found, and 500 server error. \
The 3xx range covers redirects between resources.",
    },
    CorpusPage {
        path: "howto/jwt-auth",
        title: "JWT Authentication",
        tags: "auth, security",
        body: "JWT authentication signs a token with a secret and verifies its claims and expiry. \
The server checks the token signature before trusting the claims.",
    },
    CorpusPage {
        path: "howto/oauth2-flow",
        title: "OAuth2 Authorization Code Flow",
        tags: "auth, security",
        body: "The OAuth2 authorization code flow redirects the user to grant consent. \
The client then exchanges the code for an access token at the token endpoint.",
    },
    CorpusPage {
        path: "howto/setup-tls",
        title: "Setup TLS Certificates",
        tags: "security, ops",
        body: "Setting up TLS installs a certificate chain and a private key on the server. \
The handshake negotiates a cipher and proves the certificate to the client.",
    },
    CorpusPage {
        path: "concepts/password-hashing",
        title: "Password Hashing",
        tags: "auth, security",
        body: "Password hashing stores a bcrypt or argon2 digest with a per user salt. \
The deliberately slow hash resists brute force cracking of stolen digests.",
    },
    CorpusPage {
        path: "patterns/idempotency-keys",
        title: "Idempotency Keys",
        tags: "api, resilience",
        body: "Idempotency keys let a client safely retry a request without duplicate effects. \
The server deduplicates by remembering the request id and its stored response.",
    },
    CorpusPage {
        path: "concepts/eventual-consistency",
        title: "Eventual Consistency",
        tags: "distributed",
        body: "Eventual consistency lets distributed replicas diverge briefly and then converge. \
Given no new writes all replicas eventually agree on the same value.",
    },
    CorpusPage {
        path: "concepts/cap-theorem",
        title: "CAP Theorem",
        tags: "distributed",
        body: "The CAP theorem says a partitioned system trades consistency against availability. \
Under a network partition a design must sacrifice either consistency or availability.",
    },
    CorpusPage {
        path: "patterns/saga-transactions",
        title: "Saga Pattern",
        tags: "distributed, transactions",
        body: "A saga coordinates a distributed transaction as a sequence of local steps. \
Each step has a compensating action that undoes it if a later step fails.",
    },
    CorpusPage {
        path: "patterns/outbox-pattern",
        title: "Transactional Outbox",
        tags: "distributed, events",
        body: "The transactional outbox publishes events reliably by writing them atomically with state. \
A relay reads the outbox table and forwards events after the commit.",
    },
    CorpusPage {
        path: "howto/kafka-consumer",
        title: "Kafka Consumer Groups",
        tags: "kafka, events",
        body: "A Kafka consumer group splits partitions across members and tracks the committed offset. \
A rebalance reassigns partitions when a member joins or leaves the group.",
    },
    CorpusPage {
        path: "reference/grpc-basics",
        title: "gRPC Basics",
        tags: "grpc, api",
        body: "gRPC defines services with protobuf and supports unary and streaming calls. \
The protobuf schema generates typed client and server stubs.",
    },
    CorpusPage {
        path: "concepts/rest-vs-graphql",
        title: "REST vs GraphQL",
        tags: "api",
        body: "REST endpoints can over fetch data while GraphQL lets clients select fields. \
A GraphQL resolver walks the schema to assemble exactly the requested shape.",
    },
    CorpusPage {
        path: "howto/docker-multistage",
        title: "Docker Multi-stage Build",
        tags: "docker, ops",
        body: "A Docker multi stage build compiles in a builder image and copies artifacts into a small final image. \
Separating stages minimizes the number of layers shipped to production.",
    },
    CorpusPage {
        path: "howto/k8s-deployment",
        title: "Kubernetes Deployment",
        tags: "kubernetes, ops",
        body: "A Kubernetes deployment manages replica pods and performs a rolling rollout. \
Scaling adjusts the replica count while the controller keeps pods healthy.",
    },
    CorpusPage {
        path: "patterns/blue-green-deploy",
        title: "Blue Green Deployment",
        tags: "deploy, ops",
        body: "Blue green deployment runs two environments and switches traffic for zero downtime. \
Rollback simply points traffic back at the previous stable environment.",
    },
    CorpusPage {
        path: "patterns/feature-flags",
        title: "Feature Flags",
        tags: "deploy, config",
        body: "Feature flags toggle functionality at runtime for a gradual rollout. \
Runtime configuration enables a feature for a subset of users without a deploy.",
    },
    CorpusPage {
        path: "concepts/observability",
        title: "Observability",
        tags: "observability, ops",
        body: "Observability combines metrics, logs, and traces into a coherent telemetry picture. \
The three pillars together explain system behavior under load.",
    },
    CorpusPage {
        path: "howto/prometheus-metrics",
        title: "Prometheus Metrics",
        tags: "observability, metrics",
        body: "Prometheus scrapes metric endpoints and stores counters, gauges, and histograms. \
A histogram buckets observations to estimate latency quantiles.",
    },
    CorpusPage {
        path: "concepts/distributed-tracing",
        title: "Distributed Tracing",
        tags: "observability, distributed",
        body: "Distributed tracing links spans across services using a propagated trace context. \
Each span records timing so a request path is reconstructed end to end.",
    },
    CorpusPage {
        path: "patterns/bulkhead",
        title: "Bulkhead Isolation",
        tags: "resilience",
        body: "The bulkhead pattern isolates resources into separate pools to contain failures. \
One saturated pool cannot exhaust the resources of unrelated work.",
    },
    CorpusPage {
        path: "concepts/consistent-hashing",
        title: "Consistent Hashing",
        tags: "distributed, storage",
        body: "Consistent hashing places nodes on a ring and uses virtual nodes for balance. \
Adding a node only remaps a small fraction of the keys around the ring.",
    },
    CorpusPage {
        path: "patterns/sharding",
        title: "Database Sharding",
        tags: "distributed, storage",
        body: "Database sharding horizontally partitions rows across shards by a shard key. \
Choosing a good shard key spreads load and avoids hot partitions.",
    },
    CorpusPage {
        path: "concepts/write-ahead-log",
        title: "Write Ahead Log",
        tags: "storage, durability",
        body: "A write ahead log records changes before applying them for durability. \
On crash recovery the log replays committed changes to restore state.",
    },
    CorpusPage {
        path: "concepts/mvcc",
        title: "Multi Version Concurrency Control",
        tags: "storage, transactions",
        body: "MVCC keeps multiple versions of a row to give each transaction a snapshot. \
Snapshot isolation lets readers avoid blocking writers on the same row.",
    },
    CorpusPage {
        path: "howto/index-rebuild",
        title: "Rebuild Search Index",
        tags: "search, ops",
        body: "Rebuilding the search index reindexes every document into fresh BM25 and vector structures. \
A full reindex is needed after bulk edits change many documents.",
    },
    CorpusPage {
        path: "reference/vector-database",
        title: "Vector Database",
        tags: "vectors, search",
        body: "A vector database stores embeddings and serves approximate nearest neighbor queries. \
An HNSW graph index trades a little recall for fast neighbor lookup.",
    },
    CorpusPage {
        path: "concepts/semantic-search",
        title: "Semantic Search",
        tags: "search, embeddings",
        body: "Semantic search uses dense retrieval over embeddings to match meaning not words. \
It finds relevant documents even when they share no exact query term.",
    },
    CorpusPage {
        path: "concepts/keyword-search",
        title: "Keyword Search",
        tags: "search",
        body: "Keyword search does lexical matching of exact terms against a sparse index. \
It rewards documents that contain the literal query words.",
    },
];

const GOLDEN: &[GoldenQuery] = &[
    GoldenQuery {
        query: "bm25 term frequency scoring",
        expected: &["wiki:concepts:bm25-scoring"],
    },
    GoldenQuery {
        query: "reciprocal rank fusion combine ranked lists",
        expected: &["wiki:concepts:rrf-fusion"],
    },
    GoldenQuery {
        query: "cosine similarity between vectors",
        expected: &["wiki:concepts:cosine-similarity"],
    },
    GoldenQuery {
        query: "onnx neural sentence embeddings",
        expected: &["wiki:concepts:onnx-embeddings"],
    },
    GoldenQuery {
        query: "inverted index postings list",
        expected: &["wiki:concepts:inverted-index"],
    },
    GoldenQuery {
        query: "snowball porter stemming words",
        expected: &["wiki:concepts:snowball-stemming"],
    },
    GoldenQuery {
        query: "code aware tokenizer split identifiers",
        expected: &["wiki:concepts:tokenization"],
    },
    GoldenQuery {
        query: "knowledge graph typed edges nodes",
        expected: &["wiki:concepts:knowledge-graph"],
    },
    GoldenQuery {
        query: "graph centrality inbound edges",
        expected: &["wiki:concepts:graph-centrality"],
    },
    GoldenQuery {
        query: "token bucket rate limiting requests",
        expected: &["wiki:patterns:rate-limiting"],
    },
    GoldenQuery {
        query: "circuit breaker fail fast downstream",
        expected: &["wiki:patterns:circuit-breaker"],
    },
    GoldenQuery {
        query: "exponential backoff retry jitter",
        expected: &["wiki:patterns:retry-backoff"],
    },
    GoldenQuery {
        query: "lru cache eviction policy capacity",
        expected: &["wiki:patterns:cache-eviction"],
    },
    GoldenQuery {
        query: "cache aside lazy loading on miss",
        expected: &["wiki:patterns:cache-aside"],
    },
    GoldenQuery {
        query: "redis maxmemory persistence rdb aof",
        expected: &["wiki:reference:redis-config"],
    },
    GoldenQuery {
        query: "postgres btree gin gist index planner",
        expected: &["wiki:reference:postgres-index"],
    },
    GoldenQuery {
        query: "http status codes 404 500 redirects",
        expected: &["wiki:reference:http-status-codes"],
    },
    GoldenQuery {
        query: "jwt token verify claims signature",
        expected: &["wiki:howto:jwt-auth"],
    },
    GoldenQuery {
        query: "oauth2 authorization code flow token exchange",
        expected: &["wiki:howto:oauth2-flow"],
    },
    GoldenQuery {
        query: "tls certificate chain handshake setup",
        expected: &["wiki:howto:setup-tls"],
    },
    GoldenQuery {
        query: "bcrypt argon2 password hashing salt",
        expected: &["wiki:concepts:password-hashing"],
    },
    GoldenQuery {
        query: "idempotency keys safe retry duplicate",
        expected: &["wiki:patterns:idempotency-keys"],
    },
    GoldenQuery {
        query: "eventual consistency replicas converge",
        expected: &["wiki:concepts:eventual-consistency"],
    },
    GoldenQuery {
        query: "cap theorem partition consistency availability",
        expected: &["wiki:concepts:cap-theorem"],
    },
    GoldenQuery {
        query: "saga compensating distributed transaction",
        expected: &["wiki:patterns:saga-transactions"],
    },
    GoldenQuery {
        query: "transactional outbox reliable event publishing",
        expected: &["wiki:patterns:outbox-pattern"],
    },
    GoldenQuery {
        query: "kafka consumer group offset rebalance",
        expected: &["wiki:howto:kafka-consumer"],
    },
    GoldenQuery {
        query: "grpc protobuf streaming service stubs",
        expected: &["wiki:reference:grpc-basics"],
    },
    GoldenQuery {
        query: "rest versus graphql over fetching fields",
        expected: &["wiki:concepts:rest-vs-graphql"],
    },
    GoldenQuery {
        query: "docker multi stage build small image",
        expected: &["wiki:howto:docker-multistage"],
    },
    GoldenQuery {
        query: "kubernetes deployment replica pods rollout",
        expected: &["wiki:howto:k8s-deployment"],
    },
    GoldenQuery {
        query: "blue green deployment zero downtime switch",
        expected: &["wiki:patterns:blue-green-deploy"],
    },
    GoldenQuery {
        query: "feature flags gradual rollout runtime toggle",
        expected: &["wiki:patterns:feature-flags"],
    },
    GoldenQuery {
        query: "observability metrics logs traces telemetry",
        expected: &["wiki:concepts:observability"],
    },
    GoldenQuery {
        query: "prometheus scrape counters gauges histograms",
        expected: &["wiki:howto:prometheus-metrics"],
    },
    GoldenQuery {
        query: "distributed tracing span trace context",
        expected: &["wiki:concepts:distributed-tracing"],
    },
    GoldenQuery {
        query: "bulkhead resource pool isolation failures",
        expected: &["wiki:patterns:bulkhead"],
    },
    GoldenQuery {
        query: "consistent hashing ring virtual nodes",
        expected: &["wiki:concepts:consistent-hashing"],
    },
    GoldenQuery {
        query: "database sharding shard key partition",
        expected: &["wiki:patterns:sharding"],
    },
    GoldenQuery {
        query: "write ahead log crash recovery durability",
        expected: &["wiki:concepts:write-ahead-log"],
    },
    GoldenQuery {
        query: "mvcc snapshot isolation row versions",
        expected: &["wiki:concepts:mvcc"],
    },
    GoldenQuery {
        query: "rebuild search index reindex documents",
        expected: &["wiki:howto:index-rebuild"],
    },
    GoldenQuery {
        query: "vector database approximate nearest neighbor hnsw",
        expected: &["wiki:reference:vector-database"],
    },
    GoldenQuery {
        query: "semantic search dense retrieval meaning",
        expected: &["wiki:concepts:semantic-search"],
    },
    GoldenQuery {
        query: "keyword search lexical exact term match",
        expected: &["wiki:concepts:keyword-search"],
    },
    GoldenQuery {
        query: "eviction policy when cache full",
        expected: &["wiki:patterns:cache-eviction"],
    },
    GoldenQuery {
        query: "distributed transaction across services",
        expected: &["wiki:patterns:saga-transactions"],
    },
    GoldenQuery {
        query: "document ranking relevance scoring",
        expected: &["wiki:concepts:bm25-scoring"],
    },
    GoldenQuery {
        query: "verify auth token on request",
        expected: &["wiki:howto:jwt-auth"],
    },
    GoldenQuery {
        query: "choose an index type for queries",
        expected: &["wiki:reference:postgres-index"],
    },
];

fn page_type_for(dir: &str) -> &'static str {
    match dir {
        "patterns" => "pattern",
        "reference" => "reference",
        "howto" => "howto",
        "decisions" => "decision",
        "specs" => "spec",
        "tasks" => "task",
        _ => "concept",
    }
}

fn write_corpus(root: &Path) {
    let wiki = root.join(".wm").join("wiki");
    for page in CORPUS {
        let (dir, _stem) = page
            .path
            .split_once('/')
            .expect("corpus path must be dir/stem");
        let page_type = page_type_for(dir);
        let contents = format!(
            "---\ntitle: {title}\ntype: {page_type}\ntags: [{tags}]\n---\n\n## {title}\n\n{body}\n",
            title = page.title,
            page_type = page_type,
            tags = page.tags,
            body = page.body,
        );
        let file = wiki.join(format!("{}.md", page.path));
        if let Some(parent) = file.parent() {
            std::fs::create_dir_all(parent).expect("create wiki subdir");
        }
        std::fs::write(&file, contents).expect("write corpus page");
    }
}

fn base_id(full: &str) -> &str {
    full.split('#').next().unwrap_or(full)
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "golden-query eval harness; run with --ignored --nocapture to measure recall@5"]
async fn golden_query_recall_at_5() {
    let ((_dir, root, _engine, registry), _guard) = setup_in_process().await;
    write_corpus(&root);
    call_ok(
        &registry,
        "wm_index_rebuild",
        serde_json::json!({ "skip_embed": true }),
    )
    .await;

    let mut recall_sum = 0.0;
    let mut recall_at_1_sum = 0.0;
    let mut reciprocal_rank_sum = 0.0;
    let mut misses: Vec<String> = Vec::new();

    for golden in GOLDEN {
        let resp = call_ok(
            &registry,
            "wm_search.query",
            json!({
                "q": golden.query,
                "type": "all",
                "mode": "keyword",
                "limit": RETRIEVE_LIMIT,
                "recency": false,
            }),
        )
        .await;

        let results = resp["results"].as_array().expect("results array");
        let ranked: Vec<String> = results
            .iter()
            .filter_map(|r| r["id"].as_str())
            .map(|id| base_id(id).to_string())
            .collect();
        let top: Vec<String> = ranked.iter().take(RECALL_K).cloned().collect();

        let hits = golden
            .expected
            .iter()
            .filter(|want| top.iter().any(|got| got == *want))
            .count();
        let query_recall = hits as f64 / golden.expected.len() as f64;
        recall_sum += query_recall;

        if ranked
            .first()
            .is_some_and(|got| golden.expected.contains(&got.as_str()))
        {
            recall_at_1_sum += 1.0;
        }

        if let Some(rank) = ranked
            .iter()
            .position(|got| golden.expected.contains(&got.as_str()))
        {
            reciprocal_rank_sum += 1.0 / (rank as f64 + 1.0);
        }

        if query_recall < 1.0 {
            misses.push(format!(
                "  MISS q={:?} expected={:?} top{}={:?}",
                golden.query, golden.expected, RECALL_K, top
            ));
        }
    }

    let query_count = GOLDEN.len() as f64;
    let recall = recall_sum / query_count;
    let recall_at_1 = recall_at_1_sum / query_count;
    let mrr = reciprocal_rank_sum / query_count;
    println!(
        "GOLDEN_EVAL recall@{} = {:.4} | recall@1 = {:.4} | mrr = {:.4} over {} queries",
        RECALL_K,
        recall,
        recall_at_1,
        mrr,
        GOLDEN.len()
    );
    for line in &misses {
        println!("{line}");
    }

    assert!(
        recall >= RECALL_FLOOR,
        "recall@{} {:.4} fell below floor {:.4}",
        RECALL_K,
        recall,
        RECALL_FLOOR
    );
}
