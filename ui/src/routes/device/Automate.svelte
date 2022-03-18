<script lang="ts">
    import Node from '$lib/automate/Node.svelte';
    import { key, NodeType, type Context, type NodeData, layoutStore, LayoutStore } from '$lib/automate/automate';

    import { setContext } from 'svelte';
    import { Writable, writable } from 'svelte/store';
    import Colors from '$data/colors';
import { catmull, pop } from '$data/iterators';

    const nodes: NodeData[] = [
        {
            id: 0,  
            type: NodeType.Source,
            label: "KIT_TMP",
            inputs: [],
            outputs: [
                {
                    id: 'temperature',
                    label: 'temperature °C',
                    type: { type:"numeric" },
                }
            ],
        },
        {
            id: 1,  
            type: NodeType.Sink,
            label: "KIT_BLIND",
            inputs: [
                {
                    id: 'open',
                    label: 'open %',
                    type: { type:"numeric" },
                }
            ],
            outputs: [],
        },
    ];

    const layout = new Map<number, LayoutStore>();

    let i = 0;
    for (const node of nodes) {
        const estHeight = 60 + (node.outputs.length + node.inputs.length) * 20;

        layout.set(node.id, layoutStore({
            x: (6000 + 220 * i++) - 100,
            y: 6000 - estHeight / 2,
            width: 0,
            height: 0,
        }));
    }

    const zoom = writable(1.0);
    const blockPan = writable(false);

    const map = new Map();
    const anchors = ([n, id]: [number, string]) => {
        let w: Writable<[number, number]>;
        const key = `${n}-${id}`

        if (!map.has(key)) {
            console.log('new');
            w = writable([0, 0]);
            map.set(key, w);
        } else {
            w = map.get(key);
        }

        return w;
    };

    setContext<Context>(key, { zoom, blockPan, layout, anchors });

    const axisSize = 6000;

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

        zoom.update(zoom => Math.max(min, Math.min(max, zoom - e.deltaY * sens)));        
    }

    function keyDown(e: KeyboardEvent) {
        if (e.key === ' ' && !$blockPan) spaceDown = true;
    }

    function keyUp(e: KeyboardEvent) {
        if (e.key == ' ') spaceDown = false;
    }

    function mouseDown() {
        if (!spaceDown) return;
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

    const f = anchors([0, 'temperature']);
    const l = anchors([1, 'open']);

    let d = "";

    $: {
        const [sx, sy] = $f;
        const [ex, ey] = $l;

        if (sy === ey || Math.abs(sx - ex) < 20) {
            d = `M${sx},${sy} L${ex},${ey}`;
        } else {
            let yoffset = sy > ey ? 10 : -10;
            let xoffset = sx > ex ? 10 : -10;
            let mx = sx - (sx - ex) / 2;

            d = `
                M${sx},${sy} 
                L${mx+xoffset},${sy} 
                Q${mx},${sy},${mx},${sy-yoffset} 
                L${mx},${ey+yoffset} 
                Q${mx},${ey},${mx-xoffset},${ey} 
                L${ex},${ey}
            `;
        }
    }

    $: {
        let realAxisSize = axisSize * $zoom;
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

<svelte:window 
    on:keydown={keyDown} 
    on:keyup={keyUp} 
    on:mousemove|passive={mouseMove} />

<div class="node-editor" 
     bind:clientWidth={width}
     bind:clientHeight={height}
     on:mousedown={mouseDown}
     on:mouseup={mouseUp}
     on:wheel|passive={wheel}
     class:grabbed
     class:grabenabled={spaceDown}>
     <div class="grid" style="transform: translate({panX}px, {panY}px) translate(calc(-50% + {width/2}px), calc(-50% + {height/2}px)) scale({$zoom});">
        {#each nodes as node (node.id)}
            <Node data={node} />
        {/each}

        <svg viewBox="0 0 12000 12000" class="edges">
            <g stroke-width="6" stroke-linecap="round" stroke-linejoin="round" fill="transparent" style="filter: drop-shadow(0px 0px 4px rgba(0,0,0,0.2));">
                <path d={d} stroke={Colors.device.toString()} />
            </g>

            <g>
                <rect x="5993" y="5993" width="14" height="14" rx="2" fill="rgba(0,0,0,0.18)" />
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

    .center {
        width: 12px;
        height: 12px;
        background-color: rgba(0,0,0,0.18);
        border-radius: 1px;
    }

    .edges {
        width: 12000px;
        height: 12000px;
    }
</style>