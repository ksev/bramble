<script lang="ts">
    import type Color from "color";
    import { automateContext, IOId } from "./automate";

    export let color: Color;
    export let from: IOId;
    export let to: IOId;

    const { anchors } = automateContext();

    const first = anchors(from);
    const last = anchors(to);

    let d = "";
    
    $: {      
        let distance = Math.abs($first.x - $last.x);               
        let offset = Math.max(
            60,
            (distance / 2) + 40
        );

        d = `M${$first.x},${$first.y}C${$first.x+offset},${$first.y},${$last.x-offset},${$last.y},${$last.x},${$last.y}`;
    }
</script>

<path d={d} stroke={color.alpha(0.85).toString()} />

<style>
    path { 
        mix-blend-mode: multiply; 
    }
</style>