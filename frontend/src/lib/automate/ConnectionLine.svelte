<script lang="ts">
    import colors from "$data/colors";
    import { automateContext } from "$data/automate/automate";
    import type { Connection } from "$data/automate/node";

    export let connection: Connection;
   
    const { anchors, connections } = automateContext();

    const color = colors[connection.kind.type];
    const first = anchors(connection.from);
    const last = anchors(connection.to);

    let d = "";

    function onclick() {
        connections.remove(connection);
    }
    
    $: {      
        let sqdist = $first.distanceSquared($last);

        let distance = Math.abs($first.x - $last.x);               
        let offset = Math.max(
            30,
            (distance / 2) + 30
        );

        d = sqdist < 3000 ?
            `M${$first.x},${$first.y}L${$last.x},${$last.y}` :
            `M${$first.x},${$first.y}C${$first.x+offset},${$first.y},${$last.x-offset},${$last.y},${$last.x},${$last.y}`;
    }
</script>

<path d={d} stroke={color.alpha(0.85).toString()} on:dblclick={onclick} />

<style>
    path { 
        mix-blend-mode: hard-light; 
    }
</style>