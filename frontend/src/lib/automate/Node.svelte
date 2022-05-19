<script lang="ts">
    import { automateContext, type NodeData, NodeColors } from './automate';

    import Output from './Output.svelte';
    import Input from './Input.svelte';
    import { get } from "svelte/store";
    import colors from "$data/colors";

    export let data: NodeData;

    const { zoom, layout, blockPan } = automateContext();
    const rect = layout.get(data.id);
    const singleIO = data.inputs.length === 0 || data.outputs.length === 0;
    
    let grabbing = false;
    let height = 0;

    let x = get(rect).origin.x;
    let y = get(rect).origin.y;

    function mouseDown() {
        grabbing = true;
        blockPan.set(true);
    }

    function mouseUp() {
        grabbing = false;
        blockPan.set(false);
    }

    function mouseMove(e: MouseEvent){
        if (!grabbing) return;
        
        x += e.movementX / $zoom;
        y += e.movementY / $zoom;
    }

    $: rect.resize(200, height); 
    $: rect.move(x, y);
</script>

<svelte:window on:mousemove|passive={mouseMove} on:mouseup={mouseUp} />

<div class="node" 
     bind:clientHeight|once={height} 
     style="left: {$rect.origin.x}px; top: {$rect.origin.y}px; height: {$rect.size.height || 'auto'}; width: {$rect.size.width || 200}px;">
    <h3 on:mousedown={mouseDown} 
        style="background-color: {NodeColors[data.type].toString()}"
        draggable
        class:grabbing>{data.label}</h3>
    <div class="node-body" style="background-color: {colors.background.alpha(0.9)}" class:singleIO>           
        {#each data.outputs as output (output.id)}
            <Output nodeId={data.id} data={output} />
        {/each}

        {#if data.settings}
            <div class="settings">
                <svelte:component this={data.settings} />
            </div>
        {/if}

        {#each data.inputs as input (input.id)}
            <Input nodeId={data.id} data={input} />
        {/each}
    </div>
</div>

<style>
    .node {
        position: absolute;      
        filter: drop-shadow(0px 0px 4px rgba(0,0,0,0.7));
        display: flex;
        flex-direction: column;
    }

    h3 {
        border-radius: 4px 4px 0 0;
        cursor: grab;
        color: var(--strong);        
        padding: 8px;
        font-size: 12px;
        font-weight: normal;
        cursor: grab;
        filter: drop-shadow(0px 0px 1px rgba(255,255,255,0.2));
    }

    h3.grabbing {
        cursor: grabbing;
    }

    .node-body {
        opacity: 0.9;
        display: flex;
        padding: 8px 0;
        gap: 8px;
        flex-direction: column;
        border-radius: 0 0 4px 4px;
        flex-grow: 1;
    }  

    .node-body.singleIO {
        justify-content: center;
    }

    .settings {
        flex-grow: 1;
    }   
</style>