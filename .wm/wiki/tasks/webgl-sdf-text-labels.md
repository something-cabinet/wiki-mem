---
title: WebGL SDF Text Labels with LOD
type: task
status: todo
priority: high
tags: [webgl, graph, rendering, labels]
---

## Objective
Port edge type label rendering from Canvas 2D to the WebGL (regl) renderer, with Level-of-Detail support.

## Context
The Canvas 2D renderer draws edge type labels at midpoints with rotation and a white background rect. The WebGL renderer (WebglGraphRenderer) currently has placeholder color/size buffers and no text support. WebGL text requires Signed Distance Field (SDF) rendering — a font atlas texture with per-glyph signed distance values, rendered via a fragment shader.

## Acceptance Criteria
- [ ] Edge type labels rendered in WebGL using SDF text
- [ ] Labels positioned at edge midpoint, rotated along line angle
- [ ] White background rect behind each label
- [ ] LOD levels: `k < 0.5` = none, `k >= 0.5` = priority edges, `k >= 1.0` = all
- [ ] Node color/size buffers use real data (page_type, degree) not placeholders
- [ ] Both renderer paths build clean

## Implementation Notes
- SDF font atlas: generate via `canvas` API at init time or load pre-generated
- SDF shader: sample texture, compare against threshold for anti-aliased edges
- Label geometry: instanced quads per label, positioned at edge midpoints
- Priority edge list: `extends`, `implements`, `depends_on`, `supersedes`
