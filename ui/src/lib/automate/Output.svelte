<script lang="ts">
    import { automateContext, Output } from "./automate";

    export let nodeId: number;
    export let data: Output;

    const { layout, anchors, blockPan } = automateContext();
    const nodeLayout = layout.get(nodeId);

    let anchor: HTMLDivElement;

    function mouseDown(e: MouseEvent) {
        blockPan.set(true);
    }

    function mouseUp() {
        blockPan.set(false);
    }

    $: if (anchor) {
        const key: [number, string] = [nodeId, data.id];
        // The +6 is half the width of the icon, the +1 is half width of the icon - the negative margin
        anchors(key).set([$nodeLayout.x + anchor.offsetLeft + 1, $nodeLayout.y + anchor.offsetTop + 6]);
    }
</script>

<svelte:window on:mouseup={mouseUp} />

<div class="output">
    {data.label}
    <div class="icon connected numeric" bind:this={anchor} on:mousedown={mouseDown}></div>
</div>

<style>
     .output {
        height: 20px;
        display: flex;
        justify-content: right;
        align-items: center;
        gap: 8px;

        margin-right: -5px;
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