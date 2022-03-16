<script lang="ts">
    import Node from '$lib/automate/Node.svelte';
    import { readable } from 'svelte/store';

    const axisSize = 6000;

    let x = 0;
    let y = 0;

    let panX = 0;
    let panY = 0;

    let width = 0;
    let height = 0;

    const spaceDown = readable(false, (set) => {
        const kd = (e: KeyboardEvent) => {
            if (e.key === ' ') {
                set(true);
                e.preventDefault();
            }            
        };
        const ku = (e: KeyboardEvent) => {
            if (e.key === ' ') {
                set(false);
                e.preventDefault();
            }            
        };

        document.addEventListener('keydown', kd);
        document.addEventListener('keyup', ku);

        return () => {
            document.removeEventListener('keydown', kd);
            document.removeEventListener('keyup', ku);
        }
    })

    let zoom = 1.0;
    let grabbed = false;

    let nx = 6000 - 5 * 20;
    let ny = 6000;

    let bx = 6300;
    let by = 6040;

    function changeZoom(e: WheelEvent) {
        const sens = 0.002,
              max = 3.0,
              min = (Math.min(width, height) / axisSize) * 1.5;

        zoom = Math.max(min, Math.min(max, zoom - e.deltaY * sens));        
    }

    function mouseDown() {
        if (!$spaceDown) return;
        grabbed = true;
    }

    function mouseUp() {
        grabbed = false;
    }

    function mouseMove(e: MouseEvent) {
        if (!grabbed) return;   
        
        x += e.movementX;
        y += e.movementY;
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
</script>

<div class="node-editor" 
     bind:clientWidth={width}
     bind:clientHeight={height}
     on:mousedown={mouseDown}
     on:mouseup={mouseUp}
     on:mousemove={mouseMove}
     on:wheel={changeZoom}
     class:grabbed
     class:grabenabled={$spaceDown}>
     <div class="grid" style="transform: translate({panX}px, {panY}px) translate(calc(-50% + {width/2}px), calc(-50% + {height/2}px)) scale({zoom});">
        <Node label="Temperature" bind:x={nx} bind:y={ny}  />
        <Node label="Else" bind:x={bx} bind:y={by} />

        <div class="center"></div>
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

    .center {
        width: 12px;
        height: 12px;
        background-color: rgba(0,0,0,0.18);
        border-radius: 1px;
    }
</style>