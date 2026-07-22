use fjadra::{Center, Link, ManyBody, SimulationBuilder};
use wasm_bindgen::prelude::*;

/// A handle to a running force-directed layout simulation.
/// Created from JS, advanced via `tick()`, and queried via `get_positions()`.
#[wasm_bindgen]
pub struct SimulationHandle {
    simulation: Option<fjadra::force::Simulation>,
}

#[wasm_bindgen]
impl SimulationHandle {
    /// Create a new simulation with `node_count` particles.
    ///
    /// Particles are spread in a circle around `(center_x, center_y)` with the
    /// given `spread` radius. `sources` and `targets` are parallel arrays of
    /// edge indices (length must match). Pass empty arrays for no edges.
    pub fn create(
        node_count: usize,
        center_x: f64,
        center_y: f64,
        spread: f64,
        sources: Vec<usize>,
        targets: Vec<usize>,
        link_distance: f64,
        link_strength: f64,
    ) -> Self {
        let initial_positions: Vec<[f64; 2]> = (0..node_count)
            .map(|i| {
                let angle = i as f64 * 2.399; // golden angle
                let radius = spread * (i as f64 / node_count as f64).sqrt();
                [center_x + radius * angle.cos(), center_y + radius * angle.sin()]
            })
            .collect();

        if node_count == 0 {
            return Self { simulation: None };
        }

        let edges: Vec<(usize, usize)> = sources.into_iter().zip(targets.into_iter()).collect();

        let mut sim = SimulationBuilder::default()
            .build(initial_positions)
            .add_force("center", Center::new().x(center_x).y(center_y).strength(0.3))
            .add_force("charge", ManyBody::default().strength(-200.0));

        if !edges.is_empty() {
            sim = sim.add_force("link", Link::new(edges).distance(link_distance).strength(link_strength));
        }

        Self { simulation: Some(sim) }
    }

    /// Advance the simulation by `iterations` steps.
    pub fn tick(&mut self, iterations: usize) {
        if let Some(ref mut sim) = self.simulation {
            sim.tick(iterations);
        }
    }

    /// Returns true when alpha has decayed below the minimum threshold.
    pub fn is_finished(&self) -> bool {
        self.simulation.as_ref().map_or(true, |s| s.is_finished())
    }

    /// Get current positions as a flat array `[x0, y0, x1, y1, ...]`.
    pub fn get_positions(&self) -> Vec<f64> {
        self.simulation
            .as_ref()
            .map(|s| s.positions().flat_map(|[x, y]| [x, y]).collect())
            .unwrap_or_default()
    }
}
