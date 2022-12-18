<script lang="ts">
    import colors from "$data/colors";
import type { ValueSpec } from "$data/device";
    import { value } from "$data/state";
    import Icon from "./Icon.svelte";

    export let deviceId: string;
    export let spec: ValueSpec;

    let val = value(deviceId, spec.id);

    let unit = '';

    if (spec.kind.type === 'number') {
        unit = spec.kind.unit ?? '';
    }
</script>

{#if ('Err' in $val)} 
    <div class="error" title={$val.Err}>
        <div>{$val.Err}</div>
    </div>  
{:else if ($val.Ok === null)}
    <div class="value">N/A</div> 
{:else}

    {#if (spec.kind.type === 'bool')}
        <div class="value icon">
            <Icon name={$val.Ok ? 'toggle-right' : 'toggle-left' } size={20} />
        </div>
    {:else}
        <div class="value">
            {$val.Ok}
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