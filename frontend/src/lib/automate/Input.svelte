<script lang="ts">
    import { automateContext } from "$data/automate/automate";
    import { type Slot, SlotRef } from "$data/automate/node";
    import SlotAnchor from "./SlotAnchor.svelte";
    import SlotLabel from "./SlotLabel.svelte";

    export let nodeId: number;
    export let slot: Slot;

    const id = new SlotRef(nodeId, slot.id);

    let showDefault = slot.default !== undefined;
</script>

<div class="input" class:multiple={slot.multiple}>
    <SlotAnchor id={id} multiple={slot.multiple} kind={slot.kind} direction={"input"} />

    {#if showDefault}
        <div class="default">
            <label for="default"><SlotLabel slot={slot} />:</label>
            <input id="default" value={slot.default} size="10" />
        </div>
    {:else}
        <div class="label"><SlotLabel slot={slot} /></div>
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

</style>
