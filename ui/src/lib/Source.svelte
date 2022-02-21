<script lang="ts">
    import Icon from './Icon.svelte';
    import { Color, colors } from '../colors';

    export let sensor: "source" | "sink" | "sourcesink";
    export let name: string; 

    function normalize(min: number, max: number, value: number) {
        return ((value - min) / (max - min));
    }    

    let color: Color;
    $: color = colors[sensor];

    let interval = (Math.random() * 3000) + 1000;

    let start = [];

    for (let i = 0; i < 250; i++) {
        start.push(Math.random() * 100);
    }

    let datapoints: number[] = start;

    setInterval(() => {
        datapoints = [...datapoints.slice(-249), Math.random() * 100];
    }, interval);

    let canvas: HTMLCanvasElement;
    let canvasWidth: number;
    let canvasHeight: number = 31;

    let graphColor = colors[sensor].alpha(0.6);

    $: if (canvas) {            
        let ctx = canvas.getContext("2d");
        if (datapoints.length > 0) {            
            window.requestAnimationFrame(() => {
                if (!canvas) return;
                
                if (canvas.width !== canvasWidth) {
                    canvas.width = canvasWidth;
                } 

                if (canvas.height !== canvasHeight) {
                    canvas.height = canvasHeight;
                }

                ctx.beginPath();

                const first = datapoints[0];


                ctx.moveTo(0, normalize(-50, 150, first) * canvasHeight);
            
                const step = canvasWidth / datapoints.length;

                for (let i = 1; i < datapoints.length; i++) {
                    const point = datapoints[i];
                    ctx.lineTo(step * i, normalize(-50, 150, point) * canvasHeight);
                }   

                const grad = ctx.createLinearGradient(0, 15, canvasWidth, 15);

                grad.addColorStop(0, "rgba(0,0,0,0)");
                grad.addColorStop(0.015, graphColor);
                grad.addColorStop(0.985, graphColor);
                grad.addColorStop(1.0, "rgba(0,0,0,0)");

                const fillgrad = ctx.createLinearGradient(0, 0, 0, 41);

                fillgrad.addColorStop(0, graphColor);
                fillgrad.addColorStop(1, "rgba(0,0,0,0)");

                ctx.lineWidth = 1;
                ctx.lineCap = "butt";
                ctx.lineJoin = "round";
                ctx.strokeStyle = grad;
                ctx.fillStyle = fillgrad;
                ctx.imageSmoothingQuality = "high";
                ctx.imageSmoothingEnabled = true;

                ctx.clearRect(0, 0, canvasWidth, canvasHeight);

                ctx.lineTo(canvasWidth+10, 41);
                ctx.lineTo(-10, 41);
                ctx.lineTo(0, normalize(-50, 150, first) * canvasHeight);

                ctx.stroke();

                ctx.fill();
            });            
        }
    }
</script>

<div class="item">    
    <div class="icon"><Icon name="microchip" size={16} color={color} /></div>
    <div class="name">{name}</div>
    <div class="feedback" bind:clientWidth={canvasWidth}>
        <canvas bind:this={canvas} bind:clientHeight={canvasHeight}></canvas>
    </div>
    <div class="value">{datapoints[datapoints.length-1]?.toFixed(2) ?? 'n/a'} °C</div>
    <div class="since">&lt; 1m</div>    
</div>

<style>
    .item {
        display: flex;
        align-items: stretch;
        flex-direction: row;
        gap: 3px;
        margin-top: 3px;
    }

    .name {
        padding: 6px;        
        background-color: var(--container);
    }

    .feedback {
        flex-grow: 1;
        padding: 0;
        margin: 0;
        box-sizing: border-box;
        background-color: var(--container);
        display: flex;
        justify-content: stretch;
        align-items: stretch;
    }

    .value {
        padding: 6px;

        background-color: var(--container);
        white-space: nowrap;
    }

    .since {
        padding: 6px;

        background-color: var(--container);
        white-space: nowrap;
    }

    .icon {
        padding: 6px;

        background-color: var(--container);
        display: flex;
        justify-content: center;
        align-items: center;

    }

    canvas {
        height: 31px;
        width: 100%;
        box-sizing: border-box;
    }
</style>