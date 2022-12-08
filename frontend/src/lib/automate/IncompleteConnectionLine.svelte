<script lang="ts">
    import Colors from '$data/colors';
    import { get } from 'svelte/store';
    import { automateContext, type IncompleteConnection } from './automate';

    const { anchors, pointer } = automateContext();

    export let data: IncompleteConnection;

    let d = "";

    $: if (data) {
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
    <path d={d} stroke={Colors.device.toString()} />
</g>

<style>
    path { 
        mix-blend-mode: multiply; 
    }
</style>
