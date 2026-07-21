use std::sync::Arc;

use axum::extract::State;
use axum::Json;
use serde_json::{json, Value};
use wm_core::engine::EngineState;

// ── Simple force-directed layout ─────────────────────────────────
// Implements velocity Verlet integration (same algorithm as d3-force
// and fjadra) without external dependencies. Handles 400+ node graphs
// in well under a second with direct O(n²) many-body computation.

struct Particle {
    x: f64,
    y: f64,
    vx: f64,
    vy: f64,
}

/// Run a force-directed layout simulation to convergence.
fn compute_layout(
    node_count: usize,
    edges: &[(usize, usize)],
    width: f64,
    height: f64,
    link_distance: f64,
) -> Vec<[f64; 2]> {
    // ── Initialize particles ──────────────────────────────────
    let center_x = width / 2.0;
    let center_y = height / 2.0;
    let spread = f64::min(width, height) * 0.3;

    let mut particles: Vec<Particle> = (0..node_count)
        .map(|i| {
            let angle = i as f64 * 2.399; // golden angle
            let radius = spread * (i as f64 / node_count as f64).sqrt();
            Particle {
                x: center_x + radius * angle.cos(),
                y: center_y + radius * angle.sin(),
                vx: 0.0,
                vy: 0.0,
            }
        })
        .collect();

    // ── Simulation parameters ─────────────────────────────────
    let alpha_min: f64 = 0.001;
    let alpha_decay: f64 = 1.0 - alpha_min.powf(1.0 / 300.0);
    let velocity_decay = 0.3;
    let mut alpha = 1.0;

    let many_body_strength = -200.0;
    let center_strength = 0.05;
    let link_strength = 0.3;

    // ── Simulation loop ───────────────────────────────────────
    for _ in 0..300 {
        if alpha < alpha_min {
            break;
        }

        // Reset forces
        let mut fx = vec![0.0; node_count];
        let mut fy = vec![0.0; node_count];

        // 1. Many-body force (pairwise repulsion)
        for i in 0..node_count {
            for j in (i + 1)..node_count {
                let dx = particles[j].x - particles[i].x;
                let dy = particles[j].y - particles[i].y;
                let dist_sq = dx * dx + dy * dy;
                // Clamp minimum distance to prevent explosion
                let dist_sq = dist_sq.max(1.0);
                let force = many_body_strength / dist_sq;
                let fxi = force * dx;
                let fyi = force * dy;
                fx[i] -= fxi;
                fy[i] -= fyi;
                fx[j] += fxi;
                fy[j] += fyi;
            }
        }

        // 2. Link force (spring attraction along edges)
        for &(source, target) in edges {
            let dx = particles[target].x - particles[source].x;
            let dy = particles[target].y - particles[source].y;
            let dist = (dx * dx + dy * dy).sqrt().max(1.0);
            let displacement = dist - link_distance;
            let force = link_strength * displacement / dist;
            let fxi = force * dx;
            let fyi = force * dy;
            fx[source] += fxi;
            fy[source] += fyi;
            fx[target] -= fxi;
            fy[target] -= fyi;
        }

        // 3. Center gravity (gentle pull toward viewport center)
        for i in 0..node_count {
            fx[i] -= (particles[i].x - center_x) * center_strength;
            fy[i] -= (particles[i].y - center_y) * center_strength;
        }

        // 4. Apply forces with velocity Verlet integration
        for i in 0..node_count {
            let factor = alpha * 10.0; // scale force by temperature
            particles[i].vx = (particles[i].vx + fx[i] * factor) * velocity_decay;
            particles[i].vy = (particles[i].vy + fy[i] * factor) * velocity_decay;
            particles[i].x += particles[i].vx;
            particles[i].y += particles[i].vy;
        }

        // 5. Cool down
        alpha += (0.0 - alpha) * alpha_decay;
    }

    particles.iter().map(|p| [p.x, p.y]).collect()
}

/// `POST /api/graph/layout` – Compute graph layout using force-directed placement.
pub async fn start_layout(
    State(_state): State<Arc<EngineState>>,
    Json(input): Json<Value>,
) -> Json<Value> {
    let nodes_count = input["nodes"].as_array().map(|a| a.len()).unwrap_or(0);
    let width = input["width"].as_f64().unwrap_or(800.0);
    let height = input["height"].as_f64().unwrap_or(600.0);
    let link_distance = input["link_distance"].as_f64().unwrap_or(180.0);

    // Parse edges into index pairs
    let mut edge_pairs: Vec<(usize, usize)> = Vec::new();
    if let Some(edges_arr) = input["edges"].as_array() {
        for edge_val in edges_arr {
            let source = edge_val["source"].as_u64().unwrap_or(u64::MAX) as usize;
            let target = edge_val["target"].as_u64().unwrap_or(u64::MAX) as usize;
            if source != usize::MAX && target != usize::MAX {
                edge_pairs.push((source, target));
            }
        }
    }

    let positions = if nodes_count == 0 {
        vec![[0.0_f64, 0.0_f64]; 0]
    } else {
        compute_layout(nodes_count, &edge_pairs, width, height, link_distance)
    };

    Json(json!({
        "success": true,
        "positions": positions,
    }))
}

/// `GET /api/graph/layout/{job_id}/events` – SSE stream of layout events (stub for future).
///
/// The current implementation returns positions directly from POST. This endpoint
/// exists for future two-phase streaming support but currently returns final positions
/// as a single `graph-settled` event.
pub async fn stream_events(
    State(state): State<Arc<EngineState>>,
    axum::extract::Path(_job_id): axum::extract::Path<String>,
) -> axum::response::Sse<impl tokio_stream::Stream<Item = Result<axum::response::sse::Event, std::convert::Infallible>>> {
    use axum::response::sse::Event;
    use futures::stream;

    let snapshot = state.graph.load();
    let graph = &snapshot.0;

    let mut positions: Vec<[f64; 2]> = Vec::new();
    for node_idx in graph.node_indices() {
        let i = node_idx.index() as f64;
        let angle = i * 2.399;
        let radius = 200.0 + (i * 50.0).sqrt();
        positions.push([radius * angle.cos(), radius * angle.sin()]);
    }

    let positions_json = serde_json::to_string(&json!({"positions": positions})).unwrap_or_default();

    let stream = stream::once(async move {
        Ok::<_, std::convert::Infallible>(Event::default()
            .event("graph-settled")
            .data(positions_json))
    });

    axum::response::Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(std::time::Duration::from_secs(5))
            .text("keep-alive"),
    )
}
