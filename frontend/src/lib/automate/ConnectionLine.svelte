<script lang="ts">
    import colors from "$data/colors";
    import { automateContext } from "$data/automate/automate";
    import type { Connection } from "$data/automate/node";
     import type { Readable } from "svelte/store";

    export let connection: Connection;
   
    const { anchors, connections, nodes } = automateContext();

    const inputSlot = nodes.get(connection.to.nodeId)
        .inputs.find(s => s.id === connection.to.name);

    const color = colors[inputSlot.kind.type];
    const first = anchors(connection.from);
    const last = anchors(connection.to);
    let offsetY = 0;

    let count: Readable<number>;

    if (inputSlot.multiple) {
        count = connections.connectionNumber(connection);
    }

    let d = "";

    function onclick() {
        connections.remove(connection);
    }

    $: if (count) {
        // Spread out the connection on multi inputs
        let n = $count % 7;
        let neg = n % 2 === 0;
        offsetY = neg ? n * 2 : n * -2;
    }
    
    $: {      
        let sqdist = $first.distanceSquared($last);

        let distance = Math.abs($first.x - $last.x);               
        let offset = Math.max(
            30,
            (distance / 2) + 30
        );

        d = sqdist < 3000 ?
            `M${$first.x},${$first.y}L${$last.x},${$last.y+offsetY}` :
            `M${$first.x},${$first.y}C${$first.x+offset},${$first.y},${$last.x-offset},${$last.y+offsetY},${$last.x},${$last.y+offsetY}`;
    }
</script>

<path d={d} stroke={color.alpha(0.85).toString()} on:dblclick={onclick} />

<style>
    path { 
        mix-blend-mode: hard-light; 
    }
</style>