import REGL from 'regl';
import type { GraphNode, GraphEdge } from './canvas-graph.directive';

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
  private nodeBuffer: Float32Array = new Float32Array(0);
  private edgeBuffer: Float32Array = new Float32Array(0);
  private nodeCount = 0;
  private edgeCount = 0;
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
}

/**
 * Map page_type to normalized RGB color.
 * Matches the Canvas 2D color scheme in canvas-graph.directive.ts
 */
function nodeColor(pageType: string): [number, number, number] {
  const colors: Record<string, [number, number, number]> = {
    concept: [0.231, 0.510, 0.965],   // #3b82f6
    spec: [0.133, 0.773, 0.345],      // #22c55e
    task: [0.961, 0.620, 0.043],      // #f59e0b
    memory: [0.659, 0.341, 0.965],    // #a855f7
    pattern: [0.925, 0.286, 0.600],   // #ec4899
    decision: [0.078, 0.722, 0.698],  // #14b8a6
    howto: [0.976, 0.451, 0.086],     // #f97316
    reference: [0.420, 0.451, 0.502], // #6b7280
  };
  return colors[pageType] || [0.420, 0.451, 0.502];
}

/** Map edge_type to normalized RGB color */
function edgeColor(edgeType: string): [number, number, number] {
  const colors: Record<string, [number, number, number]> = {
    extends: [0.133, 0.773, 0.345],
    implements: [0.231, 0.510, 0.965],
    depends_on: [0.961, 0.620, 0.043],
    supersedes: [0.659, 0.341, 0.965],
    references: [0.420, 0.451, 0.502],
  };
  return colors[edgeType] || [0.580, 0.580, 0.600];
}
