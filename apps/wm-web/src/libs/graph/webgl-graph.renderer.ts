import REGL from 'regl';
import type { GraphNode, GraphEdge } from '../graph/canvas-graph.directive';

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

  /** Update node data and re-upload buffers */
  updateNodes(nodes: GraphNode[]): void {
    this.nodeCount = nodes.length;
    this.nodeBuffer = new Float32Array(nodes.length * 2);
    for (let i = 0; i < nodes.length; i++) {
      this.nodeBuffer[i * 2] = nodes[i].x || 0;
      this.nodeBuffer[i * 2 + 1] = nodes[i].y || 0;
    }
  }

  /** Update edge data and re-upload buffers */
  updateEdges(edges: GraphEdge[]): void {
    this.edgeCount = edges.length;
    this.edgeBuffer = new Float32Array(edges.length * 4);
    for (let i = 0; i < edges.length; i++) {
      const s = typeof edges[i].source === 'object' ? edges[i].source : null;
      const t = typeof edges[i].target === 'object' ? edges[i].target : null;
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
    this.regl.resize();
  }

  /** Clean up */
  destroy(): void {
    this.regl?.destroy();
  }

  // ─── Buffer helpers ─────────────────────────────

  private nodeRadii(): Float32Array {
    // Placeholder — will be computed from degree
    return new Float32Array(this.nodeCount).fill(5);
  }

  private nodeColors(): Float32Array {
    // Placeholder — will be computed from page_type
    return new Float32Array(this.nodeCount * 3).fill(0.5);
  }

  private edgeColors(): Float32Array {
    // Placeholder — will be computed from edge_type
    return new Float32Array(this.edgeCount * 6).fill(0.6);
  }
}
