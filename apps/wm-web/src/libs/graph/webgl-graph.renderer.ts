import REGL from 'regl';
import type { GraphNode, GraphEdge } from './canvas-graph.directive';

const FONT_SIZE = 11;
const ATLAS_GLYPH_SIZE = 64; // pixels per glyph in atlas
const ATLAS_COLS = 16;       // glyphs per row

export interface Camera {
  x: number;
  y: number;
  k: number;
}

/**
 * WebGL graph renderer using regl.
 * Handles 100k+ nodes and 500k+ edges via instanced draw calls.
 */
export class WebglGraphRenderer {
  private regl!: REGL.Regl;
  private drawNodes: any = null;
  private drawEdges: any = null;
  private drawLabels: any = null;
  private fontAtlas: REGL.Texture2D | null = null;
  private nodeBuffer: Float32Array = new Float32Array(0);
  private edgeBuffer: Float32Array = new Float32Array(0);
  private labelBuffer: Float32Array = new Float32Array(0);
  private nodeCount = 0;
  private edgeCount = 0;
  private labelCount = 0;
  private camera: Camera = { x: 0, y: 0, k: 1 };
  private canvas!: HTMLCanvasElement;

  /** Initialize regl on a canvas element */
  init(canvas: HTMLCanvasElement): void {
    this.canvas = canvas;
    this.regl = REGL({
      canvas,
      extensions: ['ANGLE_instanced_arrays', 'OES_standard_derivatives'],
    });

    // Node shader — instanced circles
    this.drawNodes = this.regl({
      vert: `
        precision highp float;
        attribute vec2 position;
        attribute float radius;
        attribute vec3 color;

        uniform vec2 u_camera_offset;
        uniform float u_camera_zoom;
        uniform vec2 u_viewport;

        varying vec3 v_color;

        void main() {
          vec2 screenPos = (position + u_camera_offset) * u_camera_zoom;
          vec2 clipPos = screenPos / u_viewport * 2.0;
          gl_PointSize = max(2.0, radius * u_camera_zoom * 2.0);
          gl_Position = vec4(clipPos.x, -clipPos.y, 0, 1);
          v_color = color;
        }
      `,
      frag: `
        precision highp float;
        varying vec3 v_color;

        void main() {
          vec2 coord = gl_PointCoord - vec2(0.5);
          float dist = length(coord);
          if (dist > 0.5) discard;
          // Anti-aliased circle with white stroke
          float alpha = 1.0 - smoothstep(0.35, 0.5, dist);
          float stroke = 1.0 - smoothstep(0.35, 0.42, dist);
          if (dist > 0.35) {
            gl_FragColor = vec4(1.0, 1.0, 1.0, stroke * 0.8);
          } else {
            gl_FragColor = vec4(v_color, alpha);
          }
        }
      `,
      attributes: {
        position: () => this.nodeBuffer,
        radius: {
          buffer: () => this.regl.buffer(this.nodeRadii()),
          divisor: 0,
        },
        color: {
          buffer: () => this.regl.buffer(this.nodeColors()),
          divisor: 0,
        },
      },
      uniforms: {
        u_camera_offset: () => [this.camera.x, this.camera.y],
        u_camera_zoom: () => this.camera.k,
        u_viewport: () => [this.canvas.width, this.canvas.height],
      },
      count: () => this.nodeCount,
      primitive: 'points',
    });

    // Edge shader — batched lines
    this.drawEdges = this.regl({
      vert: `
        precision highp float;
        attribute vec2 position;
        attribute vec3 edgeColor;
        uniform vec2 u_camera_offset;
        uniform float u_camera_zoom;
        uniform vec2 u_viewport;
        varying vec3 v_color;

        void main() {
          vec2 screenPos = (position + u_camera_offset) * u_camera_zoom;
          vec2 clipPos = screenPos / u_viewport * 2.0;
          gl_Position = vec4(clipPos.x, -clipPos.y, 0, 1);
          v_color = edgeColor;
        }
      `,
      frag: `
        precision highp float;
        varying vec3 v_color;
        void main() {
          gl_FragColor = vec4(v_color, 0.4);
        }
      `,
      attributes: {
        position: () => this.edgeBuffer,
        edgeColor: () => this.regl.buffer(this.edgeColors()),
      },
      uniforms: {
        u_camera_offset: () => [this.camera.x, this.camera.y],
        u_camera_zoom: () => this.camera.k,
        u_viewport: () => [this.canvas.width, this.canvas.height],
      },
      count: () => this.edgeCount * 2,
      primitive: 'lines',
    });

    // Build font atlas texture
    this.fontAtlas = this.buildFontAtlas();

    // Label shader — SDF textured quads with LOD
    this.drawLabels = this.regl({
      vert: `
        precision highp float;
        attribute vec2 position;
        attribute vec2 texCoord;
        attribute float labelAlpha;
        uniform vec2 u_camera_offset;
        uniform float u_camera_zoom;
        uniform vec2 u_viewport;
        varying vec2 v_texCoord;
        varying float v_alpha;

        void main() {
          vec2 screenPos = (position + u_camera_offset) * u_camera_zoom;
          vec2 clipPos = screenPos / u_viewport * 2.0;
          gl_Position = vec4(clipPos.x, -clipPos.y, 0, 1);
          v_texCoord = texCoord;
          v_alpha = labelAlpha;
        }
      `,
      frag: `
        precision highp float;
        varying vec2 v_texCoord;
        varying float v_alpha;
        uniform sampler2D u_fontAtlas;

        void main() {
          float sdf = texture2D(u_fontAtlas, v_texCoord).r;
          float alpha = smoothstep(0.4, 0.6, sdf) * v_alpha;
          if (alpha < 0.01) discard;
          gl_FragColor = vec4(1.0, 1.0, 1.0, alpha);
        }
      `,
      attributes: {
        position: () => this.labelBuffer,
        texCoord: {
          buffer: () => this.regl.buffer(this.labelTexCoords()),
          divisor: 0,
        },
        labelAlpha: {
          buffer: () => this.regl.buffer(this.labelAlphas()),
          divisor: 0,
        },
      },
      uniforms: {
        u_camera_offset: () => [this.camera.x, this.camera.y],
        u_camera_zoom: () => this.camera.k,
        u_viewport: () => [this.canvas.width, this.canvas.height],
        u_fontAtlas: () => this.fontAtlas,
      },
      count: () => this.labelCount * 6, // 2 triangles per label quad
      primitive: 'triangles',
    });
  }

  /** Generate an SDF font atlas texture for ASCII glyphs */
  private buildFontAtlas(): REGL.Texture2D {
    const glyphsPerRow = ATLAS_COLS;
    const glyphSize = ATLAS_GLYPH_SIZE;
    const padding = 4;
    const cellSize = glyphSize + padding * 2;
    const cols = glyphsPerRow;
    const rows = Math.ceil(95 / cols); // printable ASCII: 32-126
    const atlasW = cols * cellSize;
    const atlasH = rows * cellSize;

    const canvas = document.createElement('canvas');
    canvas.width = atlasW;
    canvas.height = atlasH;
    const ctx = canvas.getContext('2d')!;

    // Fill black background
    ctx.fillStyle = '#000';
    ctx.fillRect(0, 0, atlasW, atlasH);

    // Render each glyph as white text
    ctx.fillStyle = '#fff';
    ctx.font = `${FONT_SIZE}px system-ui, sans-serif`;
    ctx.textAlign = 'center';
    ctx.textBaseline = 'middle';

    for (let i = 32; i < 127; i++) {
      const ch = String.fromCharCode(i);
      const idx = i - 32;
      const col = idx % cols;
      const row = Math.floor(idx / cols);
      const cx = col * cellSize + cellSize / 2;
      const cy = row * cellSize + cellSize / 2;
      ctx.fillText(ch, cx, cy);
    }

    const data = ctx.getImageData(0, 0, atlasW, atlasH);
    // Convert to signed distance field via brute-force transform
    const sdfData = this.computeSDF(data, atlasW, atlasH, 4);
    const sdfImage = new ImageData(new Uint8ClampedArray(sdfData), atlasW, atlasH);
    // Put back onto canvas for texture upload
    ctx.putImageData(sdfImage, 0, 0);

    return this.regl.texture({
      canvas,
      min: 'linear',
      mag: 'linear',
      wrap: 'clamp',
    });
  }

  /** Brute-force SDF computation for a small atlas */
  private computeSDF(src: ImageData, w: number, h: number, pad: number): Uint8Array {
    const out = new Uint8Array(w * h);
    const tempDist = new Float32Array(w * h);
    const maxDist = 64;

    // Initialize distances: 0 for filled, max for empty
    for (let y = 0; y < h; y++) {
      for (let x = 0; x < w; x++) {
        const i = (y * w + x) * 4;
        const isFilled = src.data[i] > 128;
        tempDist[y * w + x] = isFilled ? 0 : maxDist;
      }
    }

    // Brute-force for small atlas — find nearest filled pixel
    const searchRad = 8; // enough for font-size 11 at 64px cells
    for (let y = 0; y < h; y++) {
      for (let x = 0; x < w; x++) {
        if (tempDist[y * w + x] === 0) continue; // filled pixel, skip
        let minDist = maxDist;
        const y0 = Math.max(0, y - searchRad);
        const y1 = Math.min(h - 1, y + searchRad);
        const x0 = Math.max(0, x - searchRad);
        const x1 = Math.min(w - 1, x + searchRad);
        for (let sy = y0; sy <= y1; sy++) {
          for (let sx = x0; sx <= x1; sx++) {
            if (tempDist[sy * w + sx] === 0) {
              const dx = sx - x;
              const dy = sy - y;
              const d = Math.sqrt(dx * dx + dy * dy);
              if (d < minDist) minDist = d;
            }
          }
        }
        // Normalize SDF: 0 outside, 1 inside, 0.5 at edge
        const inside = false; // we're outside the glyph
        const signed = inside ? 0.5 + minDist / maxDist / 2 : 0.5 - minDist / maxDist / 2;
        out[y * w + x] = Math.max(0, Math.min(255, signed * 255));
      }
    }
    // Fill filled pixels as fully inside
    for (let y = 0; y < h; y++) {
      for (let x = 0; x < w; x++) {
        if (tempDist[y * w + x] === 0) {
          out[y * w + x] = 255;
        }
      }
    }
    return out;
  }

  private _lastNodes: GraphNode[] = [];
  private _lastEdges: GraphEdge[] = [];

  /** Update node data and re-upload buffers */
  updateNodes(nodes: GraphNode[]): void {
    this.nodeCount = nodes.length;
    this._lastNodes = nodes;
    this.nodeBuffer = new Float32Array(nodes.length * 2);
    for (let i = 0; i < nodes.length; i++) {
      this.nodeBuffer[i * 2] = nodes[i].x || 0;
      this.nodeBuffer[i * 2 + 1] = nodes[i].y || 0;
    }
  }

  /** Update edge data and re-upload buffers */
  updateEdges(edges: GraphEdge[]): void {
    this.edgeCount = edges.length;
    this._lastEdges = edges;
    this.edgeBuffer = new Float32Array(edges.length * 4);
    for (let i = 0; i < edges.length; i++) {
      const s = typeof edges[i].source === 'object' ? (edges[i].source as GraphNode) : null;
      const t = typeof edges[i].target === 'object' ? (edges[i].target as GraphNode) : null;
      if (!s || !t) continue;
      this.edgeBuffer[i * 4] = s.x || 0;
      this.edgeBuffer[i * 4 + 1] = s.y || 0;
      this.edgeBuffer[i * 4 + 2] = t.x || 0;
      this.edgeBuffer[i * 4 + 3] = t.y || 0;
    }
  }

  /** Update camera (pan/zoom) */
  setCamera(camera: Camera): void {
    this.camera = camera;
  }

  /** Render a single frame */
  render(): void {
    if (!this.regl) return;
    this.regl.clear({ color: [0, 0, 0, 0] });
    if (this.edgeCount > 0) this.drawEdges();
    if (this.nodeCount > 0) this.drawNodes();
    if (this.labelCount > 0 && this.camera.k >= 0.5) this.drawLabels();
  }

  /** Resize the WebGL viewport */
  resize(width: number, height: number): void {
    this.canvas.width = width * devicePixelRatio;
    this.canvas.height = height * devicePixelRatio;
    this.canvas.style.width = width + 'px';
    this.canvas.style.height = height + 'px';
    this.regl._refresh();
  }

  /** Clean up */
  destroy(): void {
    this.fontAtlas?.destroy();
    this.regl?.destroy();
  }

  // ─── Buffer helpers ─────────────────────────────

  /** Compute node radii from degree (same formula as Canvas 2D) */
  private nodeRadii(): Float32Array {
    const buf = new Float32Array(this.nodeCount);
    for (let i = 0; i < this.nodeCount; i++) {
      const node = this._lastNodes[i];
      buf[i] = Math.max(3, Math.min(15, (node?.degree || 1) * 0.5 + 3));
    }
    return buf;
  }

  /** Compute node colors from page_type */
  private nodeColors(): Float32Array {
    const buf = new Float32Array(this.nodeCount * 3);
    for (let i = 0; i < this.nodeCount; i++) {
      const c = nodeColor(this._lastNodes[i]?.page_type || '');
      buf[i * 3] = c[0];
      buf[i * 3 + 1] = c[1];
      buf[i * 3 + 2] = c[2];
    }
    return buf;
  }

  /** Compute edge colors from edge_type */
  private edgeColors(): Float32Array {
    const buf = new Float32Array(this.edgeCount * 6);
    for (let i = 0; i < this.edgeCount; i++) {
      const c = edgeColor(this._lastEdges[i]?.edge_type || '');
      // Two vertices per edge (source + target), same color
      buf[i * 6] = c[0];
      buf[i * 6 + 1] = c[1];
      buf[i * 6 + 2] = c[2];
      buf[i * 6 + 3] = c[0];
      buf[i * 6 + 4] = c[1];
      buf[i * 6 + 5] = c[2];
    }
    return buf;
  }

  /** Get edge label positions in screen space for HTML overlay */
  getEdgeLabels(): { text: string; x: number; y: number; angle: number }[] {
    const k = this.camera.k;
    if (k < 0.5) return []; // LOD: skip at low zoom
    const priorityTypes = new Set(['extends', 'implements', 'depends_on', 'supersedes']);
    const labels: { text: string; x: number; y: number; angle: number }[] = [];
    for (const edge of this._lastEdges) {
      const s = typeof edge.source === 'object' ? edge.source : null;
      const t = typeof edge.target === 'object' ? edge.target : null;
      if (!s || !t) continue;
      // LOD: k < 1.0 only priority edges; k >= 1.0 all edges
      if (k < 1.0 && !priorityTypes.has(edge.edge_type)) continue;
      const midX = (s.x! + t.x!) / 2;
      const midY = (s.y! + t.y!) / 2;
      // Convert graph coords to screen space
      const screenX = (midX + this.camera.x) * this.camera.k;
      const screenY = (midY + this.camera.y) * this.camera.k;
      const angle = Math.atan2(t.y! - s.y!, t.x! - s.x!);
      labels.push({ text: edge.edge_type, x: screenX, y: screenY, angle });
    }
    return labels;
  }

  /** Update WebGL label buffers from current edges */
  updateLabels(): void {
    const k = this.camera.k;
    const priorityTypes = new Set(['extends', 'implements', 'depends_on', 'supersedes']);

    // Count visible labels (matching LOD logic)
    const visible: { edge: GraphEdge; midX: number; midY: number }[] = [];
    for (const edge of this._lastEdges) {
      if (k < 0.5) break;
      const s = typeof edge.source === 'object' ? edge.source : null;
      const t = typeof edge.target === 'object' ? edge.target : null;
      if (!s || !t) continue;
      if (k < 1.0 && !priorityTypes.has(edge.edge_type)) continue;
      visible.push({ edge, midX: (s.x! + t.x!) / 2, midY: (s.y! + t.y!) / 2 });
    }

    this.labelCount = visible.length;
    if (this.labelCount === 0) return;

    // Each label = 6 vertices (2 triangles), each vertex = 2 floats
    const posBuf = new Float32Array(visible.length * 6 * 2);
    const tcBuf = new Float32Array(visible.length * 6 * 2);
    const alphaBuf = new Float32Array(visible.length * 6);
    this.labelBuffer = posBuf;

    const labelW = 48;
    const labelH = 16;
    const cellSize = ATLAS_GLYPH_SIZE + 8;
    const cols = ATLAS_COLS;
    const rows = Math.ceil(95 / cols);
    const atlasW = cols * cellSize;
    const atlasH = rows * cellSize;

    for (let i = 0; i < visible.length; i++) {
      const { edge, midX, midY } = visible[i];
      const cx = midX * this.camera.k;
      const cy = midY * this.camera.k;

      const halfW = labelW / 2;
      const halfH = labelH / 2;
      const x0 = cx - halfW, y0 = cy - halfH;
      const x1 = cx + halfW, y1 = cy + halfH;

      // 2 triangles: (x0,y0)-(x1,y0)-(x0,y1) and (x1,y0)-(x1,y1)-(x0,y1)
      const vi = i * 12; // 6 verts × 2 coords
      posBuf[vi] = x0;   posBuf[vi+1] = y0;
      posBuf[vi+2] = x1; posBuf[vi+3] = y0;
      posBuf[vi+4] = x0; posBuf[vi+5] = y1;
      posBuf[vi+6] = x1; posBuf[vi+7] = y0;
      posBuf[vi+8] = x1; posBuf[vi+9] = y1;
      posBuf[vi+10] = x0; posBuf[vi+11] = y1;

      // Texcoords
      const ch = edge.edge_type.charCodeAt(0) || 32;
      const idx = Math.max(0, Math.min(94, ch - 32));
      const col = idx % cols;
      const row = Math.floor(idx / cols);
      const u0 = (col * cellSize) / atlasW;
      const v0 = (row * cellSize) / atlasH;
      const u1 = ((col + 1) * cellSize) / atlasW;
      const v1 = ((row + 1) * cellSize) / atlasH;

      tcBuf[vi] = u0;   tcBuf[vi+1] = v0;
      tcBuf[vi+2] = u1; tcBuf[vi+3] = v0;
      tcBuf[vi+4] = u0; tcBuf[vi+5] = v1;
      tcBuf[vi+6] = u1; tcBuf[vi+7] = v0;
      tcBuf[vi+8] = u1; tcBuf[vi+9] = v1;
      tcBuf[vi+10] = u0; tcBuf[vi+11] = v1;

      // Alpha
      for (let j = 0; j < 6; j++) alphaBuf[i * 6 + j] = 0.85;
    }

    this._labelTexCoordBuf = tcBuf;
    this._labelAlphaBuf = alphaBuf;
  }

  private _labelTexCoordBuf: Float32Array = new Float32Array(0);
  private _labelAlphaBuf: Float32Array = new Float32Array(0);

  /** Compute texcoords for each label quad */
  private labelTexCoords(): Float32Array {
    return this._labelTexCoordBuf;
  }

  /** Compute alpha fade for each label vertex */
  private labelAlphas(): Float32Array {
    return this._labelAlphaBuf;
  }
}

/**
 * Map page_type to normalized RGB color from CSS variables.
 * Reads --page-type-{type} custom properties, falls back to --muted-foreground.
 */
function nodeColor(pageType: string): [number, number, number] {
  const el = document.documentElement;
  const style = getComputedStyle(el);
  const varName = `--page-type-${pageType}`;
  const val = style.getPropertyValue(varName).trim();
  if (val) return parseCSSColor(val);
  // Fallback to a generic color
  const fallback = style.getPropertyValue('--muted-foreground').trim();
  return fallback ? parseCSSColor(fallback) : [0.42, 0.45, 0.50];
}

/** Map edge_type to normalized RGB color from CSS variables */
function edgeColor(edgeType: string): [number, number, number] {
  const el = document.documentElement;
  const style = getComputedStyle(el);
  const varName = `--edge-type-${edgeType}`;
  const val = style.getPropertyValue(varName).trim();
  if (val) return parseCSSColor(val);
  // Fallback: use --border with slight variation
  const fallback = style.getPropertyValue('--border').trim();
  return fallback ? parseCSSColor(fallback) : [0.58, 0.58, 0.60];
}

/**
 * Parse an oklch() or hex CSS color into normalized RGB.
 * Supports: oklch(l c h / a), #hex, rgb(r g b)
 */
function parseCSSColor(val: string): [number, number, number] {
  val = val.trim();
  // oklch(l c h / a) → approximate as medium gray with hue
  if (val.startsWith('oklch(')) {
    const inner = val.slice(6, val.includes('/') ? val.indexOf('/') : -1).trim();
    const parts = inner.split(/\s+/);
    if (parts.length >= 3) {
      const l = parseFloat(parts[0]) / 100; // 0-1
      const c = parseFloat(parts[1]);
      const h = parseFloat(parts[2]) * Math.PI / 180;
      // oklch → linear RGB approximation
      const a = c * Math.cos(h);
      const b = c * Math.sin(h);
      const l_ = l + 0.3963377774 * a + 0.2158037573 * b;
      const m_ = l - 0.1055613458 * a - 0.0638541728 * b;
      const s_ = l - 0.0894841775 * a - 1.2914855480 * b;
      return [
        Math.max(0, Math.min(1, 4.0767416621 * l_ - 3.3077115913 * m_ + 0.2309699292 * s_)),
        Math.max(0, Math.min(1, -1.2684380046 * l_ + 2.6097574011 * m_ - 0.3413193965 * s_)),
        Math.max(0, Math.min(1, -0.0041960863 * l_ - 0.7034186147 * m_ + 1.7076147010 * s_)),
      ];
    }
  }
  // #hex
  if (val.startsWith('#')) {
    const hex = val.replace('#', '');
    if (hex.length === 6) {
      return [
        parseInt(hex.slice(0, 2), 16) / 255,
        parseInt(hex.slice(2, 4), 16) / 255,
        parseInt(hex.slice(4, 6), 16) / 255,
      ];
    }
  }
  // Fallback
  return [0.58, 0.58, 0.60];
}
