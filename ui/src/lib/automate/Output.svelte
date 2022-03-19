<script lang="ts">
import { Point } from "$data/geometry";

    import { automateContext, Output } from "./automate";

    export let nodeId: number;
    export let data: Output;

    const { layout, anchors, blockPan, edges } = automateContext();
    const rect = layout.get(nodeId);

    let anchor: HTMLDivElement;

    function mouseDown() {
        blockPan.set(true);
        edges.update(l => [...l, { from: [nodeId, data.id], to: "mouse", type: data.type }])
    }

    function mouseUp() {
        blockPan.set(false);
    }

    $: if (anchor) {
        const key: [number, string] = [nodeId, data.id];
        anchors(key).set(
            new Point(
                $rect.origin.x + anchor.offsetLeft + 6,
                $rect.origin.y + anchor.offsetTop + 6
            )
        );
    }
</script>

<svelte:window on:mouseup={mouseUp} />

<div class="output">
    {data.label}
    <div class="icon numeric" bind:this={anchor} on:mousedown={mouseDown}></div>
</div>

<style>
     .output {
        height: 20px;
        display: flex;
        justify-content: right;
        align-items: center;
        gap: 8px;

        margin-right: -6px;
    }

    .icon {
        background-color: var(--device);
        width: 12px;
        height: 12px;
        min-width: 12px;
        min-height: 12px;

        border: 3px solid var(--device);
        transition: 200ms linear box-shadow;        
    }

    .icon:hover, .icon.connected {
        box-shadow: 0 0 4px rgba(0,0,0,0.35) inset;
    }

    .icon.numeric {
        border-radius: 6px;
    }
</style>