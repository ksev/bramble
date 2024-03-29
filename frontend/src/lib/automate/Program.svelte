
<script lang="ts">
    import NodeBox from "$lib/automate/NodeBox.svelte";

    import { buildContext, type ContextInit } from "$data/automate/automate";
    import { Extent, Point, Rect } from "$data/geometry";
    import IncompleteConnectionLine from "$lib/automate/IncompleteConnectionLine.svelte";
    import ConnectionLine from "$lib/automate/ConnectionLine.svelte";
    import { derived, get, writable } from "svelte/store";
    import ContextMenu from "$lib/automate/ContextMenu.svelte";
    import TopMenu from "$lib/automate/TopMenu.svelte";
    import { map } from "$data/iterators";
    import Api, { ValueKind } from "$data/api";
    import { error, success } from "$data/notification";

    export let initialState: ContextInit;

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
        deviceId,
        feature,
        nodes,
        layout,
        connections,
        contextMenu,
        selected,
        blockPan,
        startedConnection,
    } = buildContext(initialState, pointer, viewPointer);

    async function save() {
        const n = nodes.all().map(n => {
            const rect = get(layout.get(n.id));

            let properties = n?.settings?.props ? 
                {
                    tag: n.properties.tag,
                    content: {
                        ...n.properties?.content,
                        ...n.settings.props,
                    }
                } : n.properties;

            return {
                id: n.id,
                properties,
                position: [
                    rect.origin.x, 
                    rect.origin.y, 
                ], 
            };
        });

        const c = Array.from(map(connections.all(), c => {
            return [
                [c.from.nodeId, c.from.name],
                [c.to.nodeId, c.to.name],
            ];
        }));

        // Extract all default values
        const defaults = nodes.all().flatMap(n => 
            n.inputs.map(s => [s.id, s.kind, s.default])
                    .filter(([_n, _k, v]: [string, ValueKind, string]) => !!v)
                    .map(([name, kind, v]) => {
                        let id = [n.id, name];
                        let val: string | boolean | number;

                        if (kind === ValueKind.Number) {
                            val = parseInt(v, 10)    
                        } else if (kind === ValueKind.Bool) {
                            val = v === 'true';
                        } else {
                            val = v;
                        }
                        
                        return [id, val];
                    })
        );

        const program = {
            counter: nodes.counter(),
            nodes: n,
            connections: c,    
            defaults,
        };

        try {
            await $Api.setAutomate({
                deviceId,
                featureId: feature,
                program
            });

            success("SAVED");
        } catch (e) {
            error(e.toString());
        }
    }

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
            translate(calc(-50% + ${width / 2}px), calc(-50% + ${height / 2}px)) 
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
    <TopMenu bind:zoom={zoom} on:home={home} on:save={save} />

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