<script lang="ts">
    import Icon from './Icon.svelte';
    import Colors, { Color } from '../data/colors';
    import Sparkline from './Sparkline.svelte';
    import { type ValueSpec } from '$data/device';
     import Value from './Value.svelte';
  
    export let spec: ValueSpec;
    export let deviceId: string;

    let color = Colors[spec.direction];
    let icon: string;

    switch (spec.direction) {
        case "sink": 
            icon = "stack-push";
            break;
        case "source":
            icon = "stack-pop";
            break;
        case "sourceSink":
            icon = "stack";
            break;
    }

    const showFeedback = spec.kind.type === 'number';

    let interval = (Math.random() * 3000) + 1000;

    let start = [];
    let num = 360;

    for (let i = 0; i < num; i++) {
        start.push(Math.random() * 100);
    }

    let datapoints: number[] = start;

    setInterval(() => {
        datapoints = [...datapoints.slice(-(num -1)), Math.random() * 100];
    }, interval);
</script>

<div class="item">    
    <div class="icon"><Icon name={icon} size={18} color={color} /></div>
    <div class="name" class:grow={!showFeedback}>{spec.name}</div>

    {#if showFeedback} 
        <div class="feedback">
            <Sparkline data={datapoints} color={color.fade(0.3)} /> 
        </div>
    {/if}
    
    {#if spec.direction !== "sink"}
        <Value deviceId={deviceId} spec={spec} />
    {/if}

    {#if spec.direction === 'sink' || spec.direction === 'sourceSink'}

    <div class="link">
        <a href={`#/device/${deviceId}/${spec.id}/automate`} title={`Automate ${spec.name}!`}>
            <Icon name="settings-automation" color={undefined}  size={20} />
        </a>
    </div>

    {/if}
</div>

<style>
    .item {
        display: flex;
        align-items: stretch;
        flex-direction: row;
        gap: 1px;
    }

    .item > div {
        border-radius: 2px;
        background-color: var(--container);
        white-space: nowrap;
    }

    .name, .link, .item > :global(.value), .item > :global(.error) {
        padding-left: 9px;   
        padding-right: 9px;
        
        display: flex;
        align-items: center;
    }

    .link:hover {
        background-color: color-mix(in srgb, var(--container) 90%, black);
    }

    .grow {
        flex-grow: 1;
    }

    .feedback {
        padding: 0;
        margin: 0;
        box-sizing: border-box;
        display: flex;
        justify-content: stretch;
        align-items: stretch;
        flex-grow: 1;
    }

    .icon {
        padding: 9px;
        display: flex;
        justify-content: center;
        align-items: center;
    }
</style>