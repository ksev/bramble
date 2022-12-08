<script lang="ts">
    import { filter } from '$data/iterators';
    import { type Device } from '$data/device';
    import Icon from './Icon.svelte';
    import Value from './Value.svelte';
    import Feature from './Feature.svelte';

    function slideNFade(node: Element, { delay = 0, duration = 400 }) {
        const o = +getComputedStyle(node).opacity;
        const w = parseInt(getComputedStyle(node).width, 10);

        return {
            delay,
            duration,
            css: (t: number) => `opacity: ${t * o}; margin-right: -${(1.0-t) * w}px`
        };
    }

    export let device: Device;

    const sources = Array.from(filter(
        device.features, 
        f => f.direction === 'source' || f.direction === 'sourceSink'
    ));

    let open = false;
</script>

<div>
    <div class="device" on:click={() => open = !open}>
        <div class="name">
            <Icon name={open ? 'chevron-down' : 'chevron-up'} size={16} />{device.name}
        </div>

        {#if !open && sources.length > 0}
            <div class="mini-view" transition:slideNFade={{ duration: 200 }}>
                {#each sources as source}
                    <Value deviceId={device.id} spec={source} />
                {/each}
            </div>
        {/if}
    </div>
    
    {#if open}
        <div class="details">
            {#each device.features as feature}
                <Feature deviceId={device.id} spec={feature} />
            {/each}
        </div>
    {/if}
</div>

<style>
    .mini-view {
        display: flex;
        align-items: stretch;
        gap: 3px;
    }

    .mini-view > :global(div) {
        padding-left: 8px;
        padding-right: 8px;
        display: flex;
        align-items: center;
    }

    .device {
        border-radius: 2px;
        display: flex;
        align-items: stretch;
        cursor: pointer;
        gap: 3px;
        
        justify-content: space-between;
        overflow: hidden;
    }

    .details {
        padding: 3px 0 0 3px;
        display: flex;
        flex-direction: column;
        gap: 3px;
    }

    .name {
        display: flex; 
        gap: 6px; 
        background-color: var(--container); 
        padding: 8px; 
        flex-grow: 1;
        transition: background-color 200ms, width 2s;
        border-radius: 2px;
        align-items: center;
    }

    .device:hover .name {
        background-color: color-mix(in srgb, var(--container) 90%, black);
    }
</style>