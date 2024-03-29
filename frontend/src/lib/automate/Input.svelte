<script lang="ts">
    import { ValueKind } from "$data/api";
    import { automateContext } from "$data/automate/automate";
    import { type Slot, SlotRef } from "$data/automate/node";
    import SlotAnchor from "./SlotAnchor.svelte";
    import SlotLabel from "./SlotLabel.svelte";

    export let nodeId: number;
    export let slot: Slot;

    const { connections } = automateContext();

    const id = new SlotRef(nodeId, slot.id);

    let connectedTo = connections.remote(id);
    let showDefault = false;

    $: {
        let hasDefault = slot.default !== undefined;
        showDefault = hasDefault && !$connectedTo;
    }
</script>

<div class="input" class:multiple={slot.multiple}>
    <SlotAnchor id={id} multiple={slot.multiple} kind={slot.kind} direction={"input"} />

    {#if showDefault && slot.kind === ValueKind.State}
        <div class="default">
            <label for="default"><SlotLabel slot={slot} />:</label>
            <select id="default" bind:value={slot.default}>
                {#each slot.meta?.possible ?? [] as n}
                    <option id={n}>{n}</option>
                {/each}
            </select>
        </div>
    {:else if showDefault && slot.kind === ValueKind.Bool }
        <div class="default">
            <label for="default"><SlotLabel slot={slot} />:</label>
            <input type="checkbox" bind:value={slot.default} />
        </div>
    {:else if showDefault}    
        <div class="default">
            <label for="default"><SlotLabel slot={slot} />:</label>
            <input id="default" bind:value={slot.default} size="10" />
        </div>
    {:else}
        <div class="label"><SlotLabel slot={slot} adaptKind={$connectedTo} /></div>
    {/if}
</div>

<style>   
    .input {
        height: 20px;
        display: flex;
        
        align-items: center;
        justify-content: left;

        margin-left: -15px;
        padding-right: 12px;
    }

    .label {
        height: 20px;
        display: flex;
        align-items: center;
        margin-left: -3px;
    }

    .input.multiple {
        height: 40px;
    }

    .default {
        border: none;
       
        display: flex;
        align-items: center;
        justify-content: space-between;
        background-color: var(--container);
        
        flex-grow: 1;
        border-radius: 4px; 
        padding: 0 6px;
        margin-left: -3px;
    }

    input {
        border: none;
        outline: none;
        background-color: unset;
        text-align: right;  
        color: #fafafa;

        border-radius: 4px;
    }

    label {
        display: flex;
        padding: 2px 0;
        color: rgba(255,255,255,0.5);
    }

    select {
        text-align: right;
    }

</style>
