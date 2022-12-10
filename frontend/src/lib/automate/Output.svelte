<script lang="ts">
    import { type Slot, SlotRef } from "$data/automate/node";
    import SlotAnchor from "./SlotAnchor.svelte";
    import SlotLabel from "./SlotLabel.svelte";

    export let nodeId: number;
    export let slot: Slot;

    const ioid = new SlotRef(nodeId, slot.id);
</script>

<div class="output">
    <div class="label"><SlotLabel slot={slot} /></div>
    <SlotAnchor id={ioid} kind={slot.kind} direction={"output"} />
</div>

<style>
     .output {
        height: 20px;
        display: flex;
        justify-content: right;
        align-items: center;

        margin-right: -15px; 
        padding-left: 12px;
    }    
    .label {
        margin-right: -3px;
        display: flex;
        justify-content: center;
        align-items: center;
    }
</style>