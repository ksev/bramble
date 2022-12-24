<script lang="ts">
    import type { Feature } from "$data/api_types";


    import { value } from "$data/state";
    import Icon from "./Icon.svelte";

    export let deviceId: string;
    export let spec: Feature;
    export let unit: string = '';

    let val = value(deviceId, spec.id);
</script>

{#if ('err' in $val)} 
    <div class="error" title={$val.err}>
        <div>{$val.err}</div>
    </div>  
{:else if ($val.ok === null)}
    <div class="value">N/A</div> 
{:else}

    {#if (spec.kind === 'BOOL')}
        <div class="value icon">
            <Icon name={$val.ok ? 'toggle-right' : 'toggle-left' } size={20} />
        </div>
    {:else}
        <div class="value">
            {$val.ok}
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
        color: rgba(255,255,255,0.3);
    }
</style>