<script lang="ts">
    import Colors from '$data/colors';
    import { get } from 'svelte/store';
    import { automateContext, type HalfEdgeData } from "./automate";

    export let data: HalfEdgeData;

    const { pointer, anchors } = automateContext();

    let d = "";

    $: if (data) {
        if ('output' in data) {
            // Giving up reactivity here does not matter
            const from = get(anchors(data.output));
            const to = data.over ? get(anchors(data.over)) : $pointer;

            d = `
                M${from.x},${from.y}
                L${to.x},${to.y}
            `;  
        } else {
            const to = get(anchors(data.input));
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
