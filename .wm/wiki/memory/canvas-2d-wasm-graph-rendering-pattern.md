---
title: Canvas 2D + WASM graph rendering pattern
type: memory
tags: [graph, canvas, wasm, layout]
status: active
---

Graph uses Canvas 2D rendering (not WebGL/regl) with fjadra WASM for force-directed layout in the browser. Edges support bezier curves for bidirectional pairs, triangle arrowheads, and HTML overlay labels. Layout runs in WASM via fjadra. Full reference: @doc/patterns/canvas2d-wasm-graph