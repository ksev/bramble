<script lang="ts">
    import { Color, kindColor } from "$data/colors";
    import { automateContext } from "$data/automate/automate";
    import type { Connection, Slot } from "$data/automate/node";

    export let connection: Connection;
   
    const { anchors, connections, nodes } = automateContext();

    const first = anchors(connection.from);
    const last = anchors(connection.to);
    let offsetY = 0;

    let color: Color;
    let inputSlot: Slot;
    let outputSlot: Slot;

    let d = "";

    function onclick() {
        connections.remove(connection);
    }

    $: {      
        inputSlot = nodes.getSlot(connection.to);
        outputSlot = nodes.getSlot(connection.from);

        color = kindColor(outputSlot.kind);

        if (inputSlot.multiple) {
            let distanceY = $first.y - $last.y;
            let clamped = Math.min(300, Math.max(-300, distanceY));
            const p = (clamped - -300) / (300 - -300);

            offsetY = p * 18 - 9;
        }

        
        
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