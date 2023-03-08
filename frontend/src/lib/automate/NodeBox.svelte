<script lang="ts">
    import { automateContext, type SelectionMove } from '$data/automate/automate';
    import type { Node } from '$data/automate/node';

    import Output from './Output.svelte';
    import Input from './Input.svelte';
    import colors from "$data/colors";
    import Icon from '$lib/Icon.svelte';
  
    export let data: Node;

    const { layout, selected, pointer, blockPan } = automateContext();
    const rect = layout.get(data.id);

    let moving: SelectionMove; 
    let height = 0;
    let width = 0;

    function mouseDown(e: MouseEvent) {
        if (!isSelected && !e.shiftKey) {
            // This is the only non sticky selection which means we will de-select after move
            selected.selectOne(data, false);
        }

        moving = selected.startMove($pointer);
        $blockPan = true;
    }
    
    function mouseUp(e: MouseEvent) {
        $blockPan = false;

        if (!moving) return;

        if (!moving.hasMoved()) {
            // We didn't move just clicked, narrow selection to this Node            
            if (!e.shiftKey) {
                // Exclusive selection 
                selected.selectOne(data);                
            } else if (isSelected) {
                // Shift is pressed and we are selected then remove this from selection
                selected.deselect(data);
            } else {
                // Shift is pressed we are not selected, extend selection with this
                selected.select(data);
            }
        } else {
            selected.deselectAll();
        }

        moving = null;
    }

    function mouseMove(){
        if (!moving) return;
        moving.move($pointer);
    }

    $: rect.resize(width, height); 

    $: nodeStyle = `
        left: ${$rect.origin.x}px; 
        top: ${$rect.origin.y}px; 
        --accent: ${data.color.alpha(1.0)};
        --body-background: ${colors.background.alpha(0.9)};
    `

    $: isSelected = $selected.has(data.id);
</script>

<svelte:window on:mousemove|passive={mouseMove} on:mouseup={mouseUp} />

<div class="node" 
     bind:clientHeight={height} 
     bind:clientWidth={width}
     on:mousedown={e => e.stopPropagation()}
     class:moving
     class:isSelected
     style={nodeStyle}>
    <div on:mousedown={mouseDown} 
        draggable
        class="title"
        class:target={data.target}>
        {data.label}
        <Icon name={data.icon} color={"white"} size={20} />
    </div>
    <div class="node-body">           
        {#each data.outputs as output (output.id)}
            <Output nodeId={data.id} slot={output} />
        {/each}

        {#if data.settings}
            <div class="settings">
                <svelte:component this={data.settings.component} {...data.settings.props} />
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
        min-width: 200px;
        background-color: var(--body-background);
        transition: 100ms filter;
        border-radius: 4px; 
    }

    .title {
        display: flex;
        justify-content: space-between;
        align-items: center;
        text-shadow: 0 0 5px rgba(0,0,0,0.15);
        cursor: grab;
        color: var(--strong);   
        border-radius: 4px 4px 0 0;     
        padding: 6px;
        font-size: 14px;
        font-weight: normal;
        cursor: grab;
        filter: drop-shadow(0px 0px 1px rgba(255,255,255,0.2));
        height: 28px;

        background-color: var(--accent);
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
        ) var(--accent); 
    }

    .moving .title {
        cursor: grabbing;
    }

    .node.isSelected {
        filter: drop-shadow(0px 0px 4px rgba(0,0,0,0.7)) 
                drop-shadow(0px 0px 2px var(--accent));
    }

    .node-body {
        display: flex;
        padding: 12px 0;
        justify-content: center;
        gap: 9px;
        flex-direction: column;
        border-radius: 0 0 4px 4px; 
        
        flex-grow: 1;
    }  

    .settings {
        flex-grow: 1;
        padding: 6px 12px;
    }   
</style>