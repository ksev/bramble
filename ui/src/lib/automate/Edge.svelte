<script lang="ts">
    import type Color from "color";
    import { automateContext } from "./automate";

    export let color: Color;
    export let from: [number, string] | "mouse";
    export let to: [number, string] | "mouse";

    const { anchors } = automateContext();

    const first = anchors(from);
    const last = anchors(to);

    let d = "";

    $: {

        d = `
            M${$first.x},${$first.y}
            L${$last.x},${$last.y}
        `;

        /*
        let offset = 20;

        let xdiff = Math.abs(sx - ey);
        let ydiff = Math.abs(sy - ey);

        let offsetX = Math.min(30, xdiff / 2);
        let offsetY = Math.min(30, ydiff / 2);
        
        let yoffset = sy > ey ? offsetY : -offsetY;
        let xoffset = sx > ex ? offsetX : -offsetX;
        let mx = sx - (sx - ex) / 2;

        d = `
            M${sx},${sy} 
            L${mx+xoffset},${sy} 
            Q${mx},${sy},${mx},${sy-yoffset} 
            L${mx},${ey+yoffset} 
            Q${mx},${ey},${mx-xoffset},${ey} 
            L${ex},${ey}
        `;  
        */      
    }
</script>

<path d={d} stroke={color.toString()} />

<style>

</style>