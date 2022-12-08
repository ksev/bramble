<script lang="ts">
    import Node from "$lib/automate/Node.svelte";

    import { buildContext, SlotRef } from "$lib/automate/automate";
    import Colors from "$data/colors";
    import { Point } from "$data/geometry";
    import IncompleteConnectionLine from "$lib/automate/IncompleteConnectionLine.svelte";
    import ConnectionLine from "$lib/automate/ConnectionLine.svelte";
    import { derived, writable } from "svelte/store";
  import { monotonic, take } from "$data/iterators";

    const zoom = writable(1.0);
    const pointer = writable<Point>(Point.ZERO);

    const {
        blockPan,
        startedConnection,
        nodes,
        connections,
    } = buildContext(
        derived(zoom, z => z), // Downstream consumers should just be able to read
        derived(pointer, p => p), // Downstream consumers should just be able to read
    );

    const cons = connections.list;

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
        startedConnection.set(null);
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

    $: transform = `
        transform: 
            translate(${panX}px, ${panY}px) 
            translate(calc(-50% + ${width / 2}px), calc(-50% + ${height / 2}px)) 
            scale(${$zoom});
    `;
</script>

<svelte:window 
    on:keydown={keyDown} 
    on:keyup={keyUp} 
    on:mousemove={mouseMove} />

<div class="node-editor"
     bind:clientWidth={width}
     bind:clientHeight={height}
     bind:this={editor}
     on:mousedown={mouseDown}
     on:mouseup={mouseUp}
     on:wheel|passive={wheel}
     class:grabbed
     class:grabenabled={spaceDown}>
    <div class="grid"style={transform}>
        {#each $nodes as node (node.id)}
            <Node data={node} />
        {/each}

        <svg viewBox="0 0 12000 12000" class="edges">
            <g>
                <rect x="5993" y="5993"
                      width="14" height="14"
                      rx="2" fill="rgba(0,0,0,0.18)" />
            </g>

            <g stroke-width="6"
               stroke-linecap="round"
               stroke-linejoin="round"
               fill="transparent"
               style="filter: drop-shadow(0px 0px 4px rgba(0,0,0,0.2));">
                {#each $cons as c (`${c.from.toString()}->${c.to.toString()}`)}
                    <ConnectionLine connection={c} color={Colors.device} />
                {/each}

                {#if $startedConnection}
                    <IncompleteConnectionLine data={$startedConnection} />
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
