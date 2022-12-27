<script lang="ts">
    import NodeBox from "$lib/automate/NodeBox.svelte";

    import { buildContext } from "$data/automate/automate";
    import { directionColor } from "$data/colors";
    import { Extent, Point, Rect } from "$data/geometry";
    import IncompleteConnectionLine from "$lib/automate/IncompleteConnectionLine.svelte";
    import ConnectionLine from "$lib/automate/ConnectionLine.svelte";
    import { derived, writable } from "svelte/store";
    import { devicesMap } from "$data/state";
    import ContextMenu from "$lib/automate/ContextMenu.svelte";
    import TopMenu from "$lib/automate/TopMenu.svelte";

    export let params: {
        deviceid: string;
        property: string;
    };

    let editor: HTMLDivElement;
    let width = 0;
    let height = 0;

    const axisSize = 6000;

    let x = 0;
    let y = 0;

    let zoom = 1.0;

    let panX = 0;
    let panY = 0;

    let selectBox: string;
    let selectBoxOrigin: Point;

    let spaceDown = false;
    let grabbed = false;

    const device = devicesMap.get(params.deviceid);
    const feature = device.features.find((f) => f.id === params.property);

    const rawPointer = writable<Point>(Point.ZERO);

    const viewPointer = derived(rawPointer, (p) => {
        const box = editor?.getBoundingClientRect();
        return box ? new Point(p.x - box.x, p.y - box.y) : Point.ZERO;
    });

    const pointer = derived(viewPointer, (p) => {
        return new Point(
            (p.x - (width / 2 + panX)) / zoom + axisSize,
            (p.y - (height / 2 + panY)) / zoom + axisSize
        );
    });

    const {
        blockPan,
        startedConnection,
        contextMenu,
        nodes,
        selected,
        connections,
    } = buildContext(
        [
            {
                id: 0,
                label: device.name,
                color: directionColor(feature.direction),
                icon: "settings-automation",
                target: true,
                inputs: [
                    {
                        id: feature.id,
                        label: `${feature.name}`,
                        kind: feature.kind,
                        meta: feature.meta,
                    },
                ],
                outputs: [],
            },
        ],
        [
            {
                id: 0,
                rect: Rect.numbers(axisSize - 100, axisSize - 40, 200, 200),
            },
        ], 
        viewPointer,
        pointer
    );

    function home() {
        x = 0;
        y = 0;
        zoom = 1.0;
    }

    function wheel(e: WheelEvent) {
        const sens = 0.001;
        zoom = zoom - e.deltaY * sens;
    }

    function keyDown(e: KeyboardEvent) {
        // If the event originates from an input, ignore it
        if ((e.target as HTMLElement)?.nodeName === "INPUT") return;
        if (e.key === " " && !$blockPan) spaceDown = true;
        if (e.ctrlKey && e.key === "a") selected.selectAll();
    }

    function keyUp(e: KeyboardEvent) {
        // If the event originates from an input, ignore it
        if ((e.target as HTMLElement)?.nodeName === "INPUT") return;

        if (e.key == " ") spaceDown = false;

        switch (e.key) {
            case " ":
                spaceDown = false;
                break;
            case "Backspace":
            case "Delete":
                const keys = Array.from($selected.keys());
                selected.deselectAll(true);

                for (const nodeId of keys) {
                    nodes.remove(nodeId);
                }
                break;
        }
    }

    function mouseDown(e: MouseEvent) {
        if (spaceDown) {
            grabbed = true;
        } else {
            selectBoxOrigin = $viewPointer;
            blockPan.set(true);
        }
    }

    function onContextMenu(e: MouseEvent) {
        e.preventDefault();
        $contextMenu = $viewPointer;
        return false;
    }

    function mouseUp() {
        grabbed = false;
        blockPan.set(false);
        startedConnection.set(null);

        selectBox = selectBoxOrigin = null;
    }

    function deselect() {
        if (selectBox || grabbed) return;
        selected.deselectAll(true);
    }

    function mouseMove(e: MouseEvent) {
        // Make the current mouse editor local mouse position availiable to all children
        rawPointer.set(new Point(e.clientX, e.clientY));

        if (selectBoxOrigin) {
            const box = Rect.corners(selectBoxOrigin, $viewPointer);

            selectBox = `
                top: ${box.origin.y}px;
                left: ${box.origin.x}px;
                width: ${box.size.width}px;
                height: ${box.size.height}px;
            `;

            selected.boxSelect(
                new Rect(
                    new Point(
                        (box.origin.x - (width / 2 + panX)) / zoom + axisSize,
                        (box.origin.y - (height / 2 + panY)) / zoom + axisSize
                    ),
                    new Extent(box.size.width / zoom, box.size.height / zoom)
                )
            );
        }

        if (grabbed) {
            x += e.movementX;
            y += e.movementY;
        }
    }

    $: {
        let realAxisSize = axisSize * zoom;
        let hwidth = width / 2;
        let hheight = height / 2;

        panX = Math.max(
            Math.min(realAxisSize - hwidth, x),
            -realAxisSize + hwidth
        );

        panY = Math.max(
            Math.min(realAxisSize - hheight, y),
            -realAxisSize + hheight
        );
    }

    $: {
        const max = 3.0;
        const min = 0.1;

        zoom = Math.max(min, Math.min(max, zoom));
        zoom = Math.round(zoom * 100) / 100;
    }

    $: transform = `
        transform: 
            translate(${panX}px, ${panY}px) 
            translate(calc(-50% + ${width / 2}px), calc(-50% + ${
        height / 2
    }px)) 
            scale(${zoom});
    `;
</script>

<svelte:window
    on:keydown={keyDown}
    on:keyup={keyUp}
    on:mouseup={mouseUp}
    on:mousemove={mouseMove}
/>

<div
    class="node-editor"
    bind:clientWidth={width}
    bind:clientHeight={height}
    bind:this={editor}
    on:mousedown={mouseDown}
    on:contextmenu={onContextMenu}
    on:wheel|passive={wheel}
    class:grabbed
    class:grabenabled={spaceDown}
>
    <TopMenu bind:zoom={zoom} on:home={home} />

    {#if $contextMenu}
        <ContextMenu />
    {/if}

    {#if selectBox}
        <div class="selectbox" style={selectBox} />
    {/if}

    <div class="grid" style={transform}>
        {#each $nodes as node (node.id)}
            <NodeBox data={node} />
        {/each}

        <svg viewBox="0 0 12000 12000" class="edges" on:mouseup|self={deselect}>
            <g>
                <rect
                    x="6003"
                    y="6003"
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
                {#each $connections as c (`${c.from.toString()}->${c.to.toString()}`)}
                    <ConnectionLine connection={c} />
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
        border-radius: 6px;
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
        background-position: -50px -50px;
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

    .selectbox {
        background-color: rgba(0, 0, 0, 0.05);
        position: absolute;
        border: 3px dashed var(--background);
        z-index: 50;
    }    
</style>
