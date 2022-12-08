<script lang="ts">
    import { automateContext, type NodeData, NodeColors } from './automate';

    import Output from './Output.svelte';
    import Input from './Input.svelte';
    import { get } from "svelte/store";
    import colors, { Color } from "$data/colors";
    import Icon from '$lib/Icon.svelte';

    export let data: NodeData;

    const { zoom, layout, blockPan } = automateContext();
    const rect = layout.get(data.id);

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
     class:grabbing
     style="left: {$rect.origin.x}px; top: {$rect.origin.y}px; height: {$rect.size.height || 'auto'}; width: {$rect.size.width || 200}px;">
    <div on:mousedown={mouseDown} 
        style="background-color: {NodeColors[data.type].toString()}"
        draggable
        class="title"
        class:target={data.target}>
        {data.label}
        <Icon name={data.icon} color={"white"} size={14} />
    </div>
    <div class="node-body" style="background-color: {colors.background.alpha(0.9)}">           
        {#each data.outputs as output (output.id)}
            <Output nodeId={data.id} slot={output} />
        {/each}

        {#if data.settings}
            <div class="settings">
                <svelte:component this={data.settings} />
            </div>
        {/if}

        {#each data.inputs as input (input.id)}
            <Input nodeId={data.id} slot={input} />
        {/each}
    </div>
</div>

<style>
    .node {
        position: absolute;      
        filter: drop-shadow(0px 0px 4px rgba(0,0,0,0.7));
        display: flex;
        flex-direction: column;
        z-index: 5;
    }

    .title {
        display: flex;
        justify-content: space-between;
        text-shadow: 0 0 5px rgba(0,0,0,0.15);
        border-radius: 4px 4px 0 0;
        cursor: grab;
        color: var(--strong);        
        padding: 8px;
        font-size: 12px;
        font-weight: normal;
        cursor: grab;
        filter: drop-shadow(0px 0px 1px rgba(255,255,255,0.2));
    }

    .title.target {
        --color:rgba(255, 255, 255, 0.1);
        background: linear-gradient(45deg, 
            transparent 0%,
            transparent 7.6923076923076925%,
            var(--color) 7.6923076923076925%,
            var(--color) 15.384615384615385%,
            transparent 15.384615384615385%,
            transparent 23.076923076923077%,
            var(--color) 23.076923076923077%,
            var(--color) 30.76923076923077%,
            transparent 30.76923076923077%,
            transparent 38.46153846153846%,
            var(--color) 38.46153846153846%,
            var(--color) 46.15384615384615%,
            transparent 46.15384615384615%,
            transparent 53.84615384615385%,
            var(--color) 53.84615384615385%,
            var(--color) 61.53846153846154%,
            transparent 61.53846153846154%,
            transparent 69.23076923076923%,
            var(--color) 69.23076923076923%,
            var(--color) 76.92307692307692%,
            transparent 76.92307692307692%,
            transparent 84.61538461538461%,
            var(--color) 84.61538461538461%,
            var(--color) 92.3076923076923%,
            transparent 92.3076923076923%,
            transparent 100%
        ); 
    }

    .grabbing .title {
        cursor: grabbing;
        
    }

    .node.grabbing {
        z-index: 6;
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

    .settings {
        flex-grow: 1;
    }   
</style>