<script lang="ts">
    import type { Feature } from "$data/api";
    import colors, { kindColor } from "$data/colors";

    import { value } from "$data/devices";
    import Icon from "./Icon.svelte";

    export let deviceId: string;
    export let spec: Feature;
    export let unit: string = '';

    let val = value(deviceId, spec.id);
    let color = kindColor(spec.kind).mix(colors.container, 0.25);
</script>

{#if ('message' in $val)} 
    <div class="error" title={$val.message}>
        <div>{$val.message}</div>
    </div>  
{:else if ($val.value === null)}
    <div class="value" style="background-color: {color}">N/A</div> 
{:else}

    {#if (spec.kind === 'BOOL')}
        <div class="value icon" style="background-color: {color}">
            <Icon name={$val.value ? 'toggle-right' : 'toggle-left' } size={20} />
        </div>
    {:else}
        <div class="value" style="background-color: {color}">
            {$val.value}
            {#if unit} 
            <span>{unit}</span>
            {/if}
        </div> 
    {/if}
{/if}

<style>
    .value {
        border-radius: 2px;
        background-color: var(--container);
        white-space: nowrap;
        display: flex;
        gap: 2px;
    }

    .error {
        white-space: nowrap;
        border-radius: 2px;
        background-color: var(--error);
        color: #fff;
        max-width: 150px;
        overflow: hidden;
    }

    .error > div {
        overflow: hidden;
        text-overflow: ellipsis;
    }

    span {
        color: rgba(255,255,255,0.4);
    }
</style>