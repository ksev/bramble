<script lang="ts">
    import { SlotRef, type Slot } from "./automate";
    import SlotAnchor from "./SlotAnchor.svelte";

    export let nodeId: string;
    export let slot: Slot;

    const ioid = new SlotRef(nodeId, slot.id);
</script>

<div class="output">
    {slot.label}
    <SlotAnchor id={ioid} kind={slot.kind} direction={"output"} />
</div>

<style>
     .output {
        height: 20px;
        display: flex;
        justify-content: right;
        align-items: center;
        gap: 8px;

        margin-right: -12px;
    }    
</style>