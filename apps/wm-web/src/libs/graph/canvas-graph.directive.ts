import { Directive, ElementRef, Input, Output, EventEmitter, OnDestroy, AfterViewInit } from '@angular/core';

export interface GraphNode {
  id: string;
  title: string;
  page_type: string;
  degree: number;
  x?: number;
  y?: number;
  fx?: number | null;
  fy?: number | null;
}

export interface GraphEdge {
  source: string | GraphNode;
  target: string | GraphNode;
  edge_type: string;
}

@Directive({
  selector: 'canvas[wmGraph]',
  standalone: true,
})
export class CanvasGraphDirective implements AfterViewInit, OnDestroy {
  @Input() nodes: GraphNode[] = [];
  @Input() edges: GraphEdge[] = [];
  @Output() nodeClick = new EventEmitter<string>();
  @Output() nodeHover = new EventEmitter<{ id: string; title: string; page_type: string; degree: number; clientX: number; clientY: number } | null>();

  private canvas!: HTMLCanvasElement;
  private ctx: CanvasRenderingContext2D | null = null;
  private resizeObserver: ResizeObserver | null = null;
  private transform = { x: 0, y: 0, k: 1 };
  private labelOverlay: HTMLDivElement | null = null;
  private labelElements: HTMLSpanElement[] = [];
  private dpr = 1;

  constructor(private el: ElementRef<HTMLCanvasElement>) {}

  ngAfterViewInit() {
    this.canvas = this.el.nativeElement;
    this.ctx = this.canvas.getContext('2d');
    this.dpr = window.devicePixelRatio || 1;
    this.setupInteraction();
    this.createLabelOverlay();
    // ResizeObserver for canvas parent
    this.resizeObserver = new ResizeObserver(() => {
      this.resizeCanvas();
    });
    if (this.canvas.parentElement) {
      this.resizeObserver.observe(this.canvas.parentElement);
    }
    this.resizeCanvas();
  }

  private resizeCanvas() {
    const parent = this.canvas.parentElement;
    if (!parent || !this.ctx) return;
    const w = parent.clientWidth;
    const h = parent.clientHeight;
    if (w === 0 || h === 0) return;
    // Set canvas size accounting for device pixel ratio
    this.canvas.width = w * this.dpr;
    this.canvas.height = h * this.dpr;
    this.canvas.style.width = w + 'px';
    this.canvas.style.height = h + 'px';
    this.ctx.setTransform(this.dpr, 0, 0, this.dpr, 0, 0);
    this.render();
  }

  private setupInteraction() {
    // Enable touch-action: none on canvas so pointer events aren't delayed on touch devices
    this.canvas.style.touchAction = 'none';

    // Track drag state per pointer
    let pointerState: {
      id: number;
      node: GraphNode | null;
      startX: number;
      startY: number;
      moved: boolean;
    } | null = null;
    let isPanning = false;
    let panStart = { x: 0, y: 0 };

    // Single unified pointerdown — replaces mousedown + touchstart
    this.canvas.addEventListener('pointerdown', (event: PointerEvent) => {
      event.preventDefault();
      this.canvas.setPointerCapture(event.pointerId);
      const [gx, gy] = this.screenToGraph(event.offsetX, event.offsetY);
      const hit = this.hitTest(gx, gy);

      pointerState = {
        id: event.pointerId,
        node: hit,
        startX: event.clientX,
        startY: event.clientY,
        moved: false,
      };

      if (hit) {
        // Pin node at current position for dragging
        hit.fx = hit.x;
        hit.fy = hit.y;
      } else {
        // Start panning
        isPanning = true;
        panStart = { x: event.clientX, y: event.clientY };
      }
    });

    this.canvas.addEventListener('pointermove', (event: PointerEvent) => {
      if (!pointerState) return;

      // Check drag threshold (3px) — below this, treat as hover/click
      const dx = event.clientX - pointerState.startX;
      const dy = event.clientY - pointerState.startY;
      if (Math.abs(dx) > 3 || Math.abs(dy) > 3) {
        pointerState.moved = true;
      }

      if (pointerState.node && pointerState.moved) {
        // Dragging a node
        const [gx, gy] = this.screenToGraph(event.offsetX, event.offsetY);
        pointerState.node.fx = gx;
        pointerState.node.fy = gy;
        this.render();
      } else if (isPanning && pointerState.moved) {
        // Panning the canvas (1:1 with mouse in screen-space)
        const pdx = event.clientX - panStart.x;
        const pdy = event.clientY - panStart.y;
        this.transform = {
          k: this.transform.k,
          x: this.transform.x + pdx,
          y: this.transform.y + pdy,
        };
        panStart = { x: event.clientX, y: event.clientY };
        this.render();
      }

      // Hover (only before drag threshold, to avoid stale hover during drag)
      if (!pointerState.moved) {
        const [hx, hy] = this.screenToGraph(event.offsetX, event.offsetY);
        const hit = this.hitTest(hx, hy);
        this.canvas.style.cursor = hit ? 'pointer' : 'grab';
        if (hit) {
          this.nodeHover.emit({ id: hit.id, title: hit.title, page_type: hit.page_type, degree: hit.degree, clientX: event.clientX, clientY: event.clientY });
        } else {
          this.nodeHover.emit(null);
        }
      }
    });

    this.canvas.addEventListener('pointerup', (event: PointerEvent) => {
      if (!pointerState) return;

      // Click on node (no movement) — navigate
      if (pointerState.node && !pointerState.moved) {
        this.nodeClick.emit(pointerState.node.id);
      }

      // Unpin dragged node (keep in final position)
      if (pointerState.node) {
        pointerState.node.fx = pointerState.node.x;
        pointerState.node.fy = pointerState.node.y;
        this.render();
      }

      pointerState = null;
      isPanning = false;
    });

    this.canvas.addEventListener('pointercancel', () => {
      pointerState = null;
      isPanning = false;
    });

    // Wheel zoom — manual (doesn't conflict with pointer events)
    this.canvas.addEventListener('wheel', (event: WheelEvent) => {
      event.preventDefault();
      const delta = event.deltaY > 0 ? 1 / 1.1 : 1.1;
      const newK = this.transform.k * delta;
      if (newK < 0.1 || newK > 4) return;

      // Zoom centered on mouse position (mx, my) in screen space
      const mx = event.offsetX;
      const my = event.offsetY;
      const newX = mx * (1 - delta) + delta * this.transform.x;
      const newY = my * (1 - delta) + delta * this.transform.y;
      this.transform = { x: newX, y: newY, k: newK };
      this.render();
    }, { passive: false });

    // Double-tap on a node to unpin it (timing-based, replaces dblclick)
    let lastTap = 0;
    this.canvas.addEventListener('pointerdown', (_event: PointerEvent) => {
      const now = Date.now();
      if (now - lastTap < 300 && pointerState?.node) {
        pointerState.node.fx = null as any;
        pointerState.node.fy = null as any;
      }
      lastTap = now;
    });
  }

  /** Convert screen coordinates to graph space (accounts for zoom/pan transform) */
  private screenToGraph(sx: number, sy: number): [number, number] {
    return [
      (sx - this.transform.x) / this.transform.k,
      (sy - this.transform.y) / this.transform.k,
    ];
  }

  /** Compute node radius from degree — uses sqrt scaling for natural distribution */
  private nodeRadius(node: GraphNode): number {
    return Math.max(18, Math.min(55, Math.sqrt(node.degree || 1) * 8 + 10));
  }

  /** Find node at graph coordinates, returns null if none */
  private hitTest(gx: number, gy: number): GraphNode | null {
    for (const node of this.nodes) {
      const dx = gx - node.x!;
      const dy = gy - node.y!;
      const r = this.nodeRadius(node);
      if (dx * dx + dy * dy < (r + 8) * (r + 8)) return node;
    }
    return null;
  }

  /** Zoom in/out by a factor (e.g., 1.3 to zoom in, 1/1.3 to zoom out) */
  zoomBy(factor: number) {
    const newK = this.transform.k * factor;
    if (newK < 0.1 || newK > 4) return;
    // Zoom centered on canvas center
    const cx = this.canvas.clientWidth / 2;
    const cy = this.canvas.clientHeight / 2;
    const newX = cx * (1 - factor) + factor * this.transform.x;
    const newY = cy * (1 - factor) + factor * this.transform.y;
    this.transform = { x: newX, y: newY, k: newK };
    this.render();
  }

  /** Fit graph to viewport */
  fitToView() {
    if (this.nodes.length === 0) return;
    const parent = this.canvas.parentElement!;
    const w = parent.clientWidth;
    const h = parent.clientHeight;
    if (w === 0 || h === 0) return;

    let minX = Infinity, minY = Infinity, maxX = -Infinity, maxY = -Infinity;
    for (const node of this.nodes) {
      if (node.x === undefined || node.y === undefined) continue;
      const r = this.nodeRadius(node);
      minX = Math.min(minX, node.x - r);
      minY = Math.min(minY, node.y - r);
      maxX = Math.max(maxX, node.x + r);
      maxY = Math.max(maxY, node.y + r);
    }
    if (!isFinite(minX)) return;

    const bbw = maxX - minX;
    const bbh = maxY - minY;
    const padding = 60;
    const maxK = 2;
    const k = Math.min(maxK, Math.min((w - padding * 2) / (bbw || 1), (h - padding * 2) / (bbh || 1)));
    const cx = (minX + maxX) / 2;
    const cy = (minY + maxY) / 2;

    this.transform = { x: w / 2 - k * cx, y: h / 2 - k * cy, k };
    this.render();
  }

  private render() {
    if (!this.ctx) return;
    const ctx = this.ctx;
    const canvas = this.canvas;
    const w = canvas.width / this.dpr;
    const h = canvas.height / this.dpr;

    // Clear canvas
    ctx.save();
    ctx.setTransform(this.dpr, 0, 0, this.dpr, 0, 0);
    ctx.clearRect(0, 0, w, h);

    // Apply camera transform
    ctx.translate(this.transform.x, this.transform.y);
    ctx.scale(this.transform.k, this.transform.k);

    // Draw edges
    for (const edge of this.edges) {
      const source = typeof edge.source === 'object' ? edge.source : this.nodes.find(n => n.id === edge.source);
      const target = typeof edge.target === 'object' ? edge.target : this.nodes.find(n => n.id === edge.target);
      if (!source || !target || source.x === undefined || target.x === undefined) continue;

      const color = this.readCssColor(`--edge-type-${edge.edge_type}`, 0.6) || 'oklch(0.5 0.05 0 / 0.6)';
      ctx.beginPath();
      ctx.moveTo(source.x, source.y);
      ctx.lineTo(target.x, target.y);
      ctx.strokeStyle = color;
      ctx.lineWidth = 1.5 / this.transform.k;
      ctx.stroke();
    }

    // Draw nodes
    for (const node of this.nodes) {
      if (node.x === undefined) continue;
      const radius = this.nodeRadius(node) / this.transform.k;
      const color = this.readCssColor(`--page-type-${node.page_type}`, 0.85) || 'oklch(0.5 0.05 0 / 0.85)';

      // White stroke outline
      ctx.beginPath();
      ctx.arc(node.x, node.y, radius + 1.5 / this.transform.k, 0, Math.PI * 2);
      ctx.fillStyle = 'rgba(255, 255, 255, 0.3)';
      ctx.fill();

      // Colored fill
      ctx.beginPath();
      ctx.arc(node.x, node.y, radius, 0, Math.PI * 2);
      ctx.fillStyle = color;
      ctx.fill();
    }

    ctx.restore();

    this.updateLabelOverlay();
  }

  /** Create HTML overlay for edge labels in WebGL mode */
  private createLabelOverlay(): void {
    this.labelOverlay = document.createElement('div');
    this.labelOverlay.className = 'graph-label-overlay';
    this.labelOverlay.style.cssText = `
      position: absolute; top: 0; left: 0; width: 100%; height: 100%;
      pointer-events: none; overflow: hidden;
    `;
    this.canvas.parentElement?.style.setProperty('position', 'relative');
    this.canvas.insertAdjacentElement('afterend', this.labelOverlay);
  }

  /** Compute edge label positions for the HTML overlay */
  private getEdgeLabels(): { text: string; x: number; y: number; angle: number }[] {
    const labels: { text: string; x: number; y: number; angle: number }[] = [];
    for (const edge of this.edges) {
      const source = typeof edge.source === 'object' ? edge.source : this.nodes.find(n => n.id === edge.source);
      const target = typeof edge.target === 'object' ? edge.target : this.nodes.find(n => n.id === edge.target);
      if (!source || !target || source.x === undefined || target.x === undefined) continue;

      const mx = (source.x + target.x) / 2;
      const my = (source.y + target.y) / 2;
      const angle = Math.atan2(target.y - source.y, target.x - source.x);
      // Apply camera transform to get screen coordinates
      const sx = mx * this.transform.k + this.transform.x;
      const sy = my * this.transform.k + this.transform.y;
      labels.push({ text: edge.edge_type.replace(/_/g, ' '), x: sx, y: sy, angle });
    }
    return labels;
  }

  /** Update edge label positions in the HTML overlay */
  private updateLabelOverlay(): void {
    if (!this.labelOverlay) return;
    const labels = this.getEdgeLabels();
    // Rebuild label elements if count changed
    while (this.labelElements.length < labels.length) {
      const el = document.createElement('span');
      const labelBg = this.readCssColor('--card', 0.9);
      const labelFg = this.readCssColor('--muted-foreground', 0.85);
      el.style.cssText = `
        position: absolute; font: 9px sans-serif; white-space: nowrap;
        pointer-events: none; transform-origin: center center;
        background: ${labelBg}; padding: 1px 3px; border-radius: 2px;
        color: ${labelFg}; line-height: 1;
      `;
      this.labelOverlay.appendChild(el);
      this.labelElements.push(el);
    }
    // Remove excess elements
    while (this.labelElements.length > labels.length) {
      const el = this.labelElements.pop()!;
      el.remove();
    }
    // Position each label
    const parent = this.canvas.parentElement!;
    const w = parent.clientWidth;
    const h = parent.clientHeight;
    for (let i = 0; i < labels.length; i++) {
      const l = labels[i];
      const el = this.labelElements[i];
      el.textContent = l.text;
      el.style.transform = `translate(${l.x}px, ${l.y}px) rotate(${l.angle}rad) translate(-50%, -50%)`;
      // Hide if off-screen
      if (l.x < -50 || l.x > w + 50 || l.y < -50 || l.y > h + 50) {
        el.style.display = 'none';
      } else {
        el.style.display = '';
      }
    }
  }

  /** Read a CSS custom property value with optional alpha */
  private readCssColor(name: string, alpha?: number): string {
    const color = getComputedStyle(document.documentElement).getPropertyValue(name).trim();
    if (alpha !== undefined && color.startsWith('oklch(')) {
      const inner = color.slice(6, -1);
      return `oklch(${inner} / ${alpha})`;
    }
    return color;
  }

  triggerRender(): void {
    this.render();
  }

  ngOnDestroy() {
    this.resizeObserver?.disconnect();
    this.labelOverlay?.remove();
  }
}
