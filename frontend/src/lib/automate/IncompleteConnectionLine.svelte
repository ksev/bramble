<script lang="ts">
    import colors from '$data/colors';
    import { Color } from '$data/colors';
    import { get } from 'svelte/store';
    import { automateContext } from '$data/automate/automate';
    import type { IncompleteConnection } from '$data/automate/node';

    const { anchors, pointer } = automateContext();

    export let data: IncompleteConnection;
    
    let color: Color;

    let d = "";

    $: if (data) {
        color = colors[data.kind.type];

        if (data.startDirection === 'output') {
            // Giving up reactivity here does not matter
            const from = get(anchors(data.start));
            const to = data.over ? get(anchors(data.over)) : $pointer;

            d = `
                M${from.x},${from.y}
                L${to.x},${to.y}
            `;  
        } else {
            const to = get(anchors(data.start));
            const from = data.over ? get(anchors(data.over)) : $pointer;

            d = `                
                M${from.x},${from.y}
                L${to.x},${to.y}
            `;  
        }           
    }
</script>

<g>
    <path d={d} stroke={color.alpha(0.85).toString()} />
</g>

<style>
    path { 
        mix-blend-mode: hard-light; 
    }
</style>
