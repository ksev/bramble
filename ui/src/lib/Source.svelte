<script lang="ts">
    import Icon from './Icon.svelte';
    import Colors, { Color } from '../colors';
    import Sparkline from './Sparkline.svelte';

    export let sensor: "source" | "sink" | "sourcesink";
    export let name: string; 

    let color = Colors[sensor];

    let interval = (Math.random() * 3000) + 1000;

    let start = [];
    let num = 150;

    for (let i = 0; i < num; i++) {
        start.push(Math.random() * 100);
    }

    let datapoints: number[] = start;

    setInterval(() => {
        datapoints = [...datapoints.slice(-(num -1)), Math.random() * 100];
    }, interval);
</script>

<div class="item">    
    <div class="icon"><Icon name="microchip" size={16} color={color} /></div>
    <div class="name">{name}</div>
    <div class="feedback">
        <Sparkline data={datapoints} color={color.fade(0.3)} /> 
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
</style>