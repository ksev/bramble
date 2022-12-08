<script lang="ts">
    import type Color from "color";
    import { automateContext, type Connection } from "./automate";

    export let color: Color;
    export let connection: Connection;
   
    let { to, from } = connection;
    const { anchors, connections } = automateContext();

    const first = anchors(from);
    const last = anchors(to);

    let d = "";

    function onclick() {
        connections.remove(connection);
    }
    
    $: {      
        let distance = Math.abs($first.x - $last.x);               
        let offset = Math.max(
            60,
            (distance / 2) + 60
        );

        d = distance < 60 ?
            `M${$first.x},${$first.y}L${$last.x},${$last.y}` :
            `M${$first.x},${$first.y}C${$first.x+offset},${$first.y},${$last.x-offset},${$last.y},${$last.x},${$last.y}`;
    }
</script>

<path d={d} stroke={color.alpha(0.85).toString()} on:dblclick={onclick} />

<style>
    path { 
        mix-blend-mode: multiply; 
    }
</style>