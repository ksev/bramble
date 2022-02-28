<script lang="ts">
import { pop } from 'svelte-spa-router';

    import Colors, { Color } from '../data/colors';
    import * as Iter from '../data/iterators';

    export let data: number[];
    export let color: Color;
    export let tension: number = 0.5;

    let containerWidth = 0;
    let containerHeight = 0;

    function progress(min: number, max: number, value: number) {
        return ((value - min) / (max - min));
    }    

    let min = 0;
    let max = 0;

    $: {
        min = Number.POSITIVE_INFINITY;
        max = Number.NEGATIVE_INFINITY;

        for (let i = 0; i < data.length; i++) {
            const v = data[i];

            if (v < min) min = v;
            if (v > max) max = v;
        }

        let padding = (max - min) * 0.2;

        max = max + padding;
        min = min - padding;
    }

    let canvas: HTMLCanvasElement;

    $: if (canvas && data.length > 0) {            
        window.requestAnimationFrame(() => {
            if (!canvas) return;

            if (canvas.width !== containerWidth) canvas.width = containerWidth;
            if (canvas.height !== containerHeight) canvas.height = containerHeight;

            let ctx = canvas.getContext("2d");

            const grad = ctx.createLinearGradient(0, 0, containerWidth, 0);

            grad.addColorStop(0, Colors.transparent.toString());
            grad.addColorStop(0.015, color.toString());
            grad.addColorStop(0.985, color.toString());
            grad.addColorStop(1.0, Colors.transparent.toString());

            const fillgrad = ctx.createLinearGradient(0, 0, 0, containerHeight+1);

            fillgrad.addColorStop(0, color.toString());
            fillgrad.addColorStop(1, Colors.transparent.toString());

            ctx.lineWidth = 1;
            ctx.lineCap = "butt";
            ctx.lineJoin = "round";
            ctx.strokeStyle = grad;
            ctx.fillStyle = fillgrad;
            ctx.imageSmoothingQuality = "high";
            ctx.imageSmoothingEnabled = true;

            ctx.beginPath();

            const step = containerWidth / data.length;

            const coords = Iter.map(data, 
                (n, i): [number, number] => [
                    step * i, 
                    progress(min, max, n) * containerHeight
                ]
            );

            const spline = Iter.catmull(coords, data.length, tension);

            const [[firstX, firstY]] = Iter.pop(spline);

            ctx.moveTo(firstX, firstY);
                    
            for (const [[x,y], [cp1x, cp1y], [cp2x, cp2y]] of spline) {
                ctx.bezierCurveTo(cp1x, cp1y, cp2x, cp2y, x, y);
            }   
                        
            ctx.clearRect(0, 0, containerWidth, containerHeight);

            ctx.lineTo(containerWidth+1, containerHeight+1);
            ctx.lineTo(-1, containerHeight+1);

            ctx.lineTo(firstX, firstY);

            ctx.stroke();
            ctx.fill();
        });            
    }
</script>

<div bind:clientWidth={containerWidth} bind:clientHeight={containerHeight}>
    {#if containerWidth && containerHeight}
        <canvas bind:this={canvas}></canvas>
    {/if}
</div>

<style>
    div, canvas {
        width: 100%;
        height: 100%;

        box-sizing: border-box;
    }
</style>