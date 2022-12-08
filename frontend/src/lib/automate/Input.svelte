<script lang="ts">
    import { type Slot, SlotRef } from "./automate";
    import SlotAnchor from "./SlotAnchor.svelte";

    export let nodeId: string;
    export let slot: Slot;

    const ioid = new SlotRef(nodeId, slot.id);
</script>

<div class="input" class:multiple={slot.multiple}>
    <SlotAnchor id={ioid} multiple={slot.multiple} kind={slot.kind} direction={"input"} />
    {slot.label}
</div>

<style>   
    .input {
        height: 20px;
        display: flex;
        justify-content: left;
        align-items: center;
        gap: 8px;

        margin-left: -12px;
    }

    .input.multiple {
        height: 44px;
    }

</style>
