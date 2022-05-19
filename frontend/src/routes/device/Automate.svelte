<script lang="ts">
  import Node from "$lib/automate/Node.svelte";
  import Edge from "$lib/automate/Edge.svelte";

  import {
    key,
    NodeType,
    type Context,
    type NodeData,
    layoutStore,
    type LayoutStore,
    type EdgeData,
    type HalfEdgeData,
    IOId,
  } from "$lib/automate/automate";

  import { setContext } from "svelte";
  import { type Writable, writable } from "svelte/store";
  import Colors from "$data/colors";
  import { Extent, Point, Rect } from "$data/geometry";
  import HalfEdge from "$lib/automate/HalfEdge.svelte";

  const nodes: NodeData[] = [
    {
      id: 0,
      type: NodeType.Source,
      label: "KIT_TMP",
      inputs: [],
      outputs: [
        {
          id: "temperature_c",
          label: "temperature °C",
          type: { kind: "numeric" },
        },
        {
          id: "temperature_f",
          label: "temperature °F",
          type: { kind: "numeric" },
        },
      ],
    },
    {
      id: 1,
      type: NodeType.Sink,
      label: "KIT_BLIND",
      inputs: [
        {
          id: "open",
          label: "open %",
          type: { kind: "numeric" },
        },
      ],
      outputs: [],
    },
  ];

  const layout = new Map<number, LayoutStore>();

  let i = 0;
  for (const node of nodes) {
    const estHeight = 60 + (node.outputs.length + node.inputs.length) * 20;

    layout.set(
      node.id,
      layoutStore(
        new Rect(
          new Point(6000 + 220 * i++ - 100, 6000 - estHeight / 2),
          new Extent(0, 0)
        )
      )
    );
  }

  const zoom = writable(1.0);
  const blockPan = writable(false);
  const edges = writable<EdgeData[]>([]);
  const pointer = writable<Point>(Point.ZERO);
  const halfEdge = writable<HalfEdgeData>(null);

  const map = new Map();
  const anchors = (key: IOId): Writable<Point> => {
    let w: Writable<Point>;
    let strKey = key.toString();

    if (!map.has(strKey)) {
      w = writable(Point.ZERO);
      map.set(strKey, w);
    } else {
      w = map.get(strKey);
    }

    return w;
  };

  setContext<Context>(key, {
    zoom,
    blockPan,
    pointer,

    layout,
    anchors,
    edges,
    halfEdge,
  });

  const axisSize = 6000;
  let editor: HTMLDivElement;

  let x = 0;
  let y = 0;

  let panX = 0;
  let panY = 0;

  let width = 0;
  let height = 0;

  let spaceDown = false;
  let grabbed = false;

  function wheel(e: WheelEvent) {
    const sens = 0.002,
      max = 3.0,
      min = (Math.min(width, height) / axisSize) * 1.5;

    zoom.update((zoom) => Math.max(min, Math.min(max, zoom - e.deltaY * sens)));
  }

  function keyDown(e: KeyboardEvent) {
    if (e.key === " " && !$blockPan) spaceDown = true;
  }

  function keyUp(e: KeyboardEvent) {
    if (e.key == " ") spaceDown = false;
  }

  function mouseDown() {
    if (!spaceDown) return;
    grabbed = true;
  }

  function mouseUp() {
    grabbed = false;
    blockPan.set(false);
    halfEdge.set(null);
  }

  function mouseMove(e: MouseEvent) {
    const box = editor.getBoundingClientRect();

    // Make the current mouse editor local mouse position availiable to all children
    pointer.set(
      new Point(
        (e.clientX - box.x - (width / 2 + panX)) / $zoom + axisSize,
        (e.clientY - box.y - (height / 2 + panY)) / $zoom + axisSize
      )
    );

    if (!grabbed) return;

    x += e.movementX;
    y += e.movementY;
  }

  $: {
    let realAxisSize = axisSize * $zoom;
    let hwidth = width / 2;
    let hheight = height / 2;

    panX = Math.max(Math.min(realAxisSize - hwidth, x), -realAxisSize + hwidth);

    panY = Math.max(
      Math.min(realAxisSize - hheight, y),
      -realAxisSize + hheight
    );
  }
</script>

<svelte:window on:keydown={keyDown} on:keyup={keyUp} on:mousemove={mouseMove} />

<div
  class="node-editor"
  bind:clientWidth={width}
  bind:clientHeight={height}
  bind:this={editor}
  on:mousedown={mouseDown}
  on:mouseup={mouseUp}
  on:wheel|passive={wheel}
  class:grabbed
  class:grabenabled={spaceDown}
>
  <div
    class="grid"
    style="transform: translate({panX}px, {panY}px) translate(calc(-50% + {width /
      2}px), calc(-50% + {height / 2}px)) scale({$zoom});"
  >
    {#each nodes as node (node.id)}
      <Node data={node} />
    {/each}

    <svg viewBox="0 0 12000 12000" class="edges">
      <g>
        <rect
          x="5993"
          y="5993"
          width="14"
          height="14"
          rx="2"
          fill="rgba(0,0,0,0.18)"
        />
      </g>

      <g
        stroke-width="6"
        stroke-linecap="round"
        stroke-linejoin="round"
        fill="transparent"
        style="filter: drop-shadow(0px 0px 4px rgba(0,0,0,0.2));"
      >
        {#each $edges as edge}
          <Edge from={edge.output} to={edge.input} color={Colors.device} />
        {/each}

        {#if $halfEdge}
          <HalfEdge data={$halfEdge} />
        {/if}
      </g>
    </svg>
  </div>
</div>

<style>
  .node-editor {
    background: var(--container);
    padding: 0;
    margin: 0;
    border-radius: 4px;
    position: relative;
    overflow: hidden;

    -webkit-user-select: none;
    -moz-user-select: none;
    -ms-user-select: none;
    user-select: none;

    display: flex;
    flex-direction: column;
    height: 100%;
    width: 100%;
  }

  .node-editor .grid {
    background-image: url(/grid.svg);
    background-position: -60px -60px;
    width: 12000px;
    height: 12000px;

    position: absolute;

    display: flex;
    justify-content: center;
    align-items: center;
  }

  .node-editor.grabenabled {
    cursor: grab;
  }

  .node-editor.grabbed {
    cursor: grabbing !important;
  }

  .edges {
    width: 12000px;
    height: 12000px;
  }
</style>
