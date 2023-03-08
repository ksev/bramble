<script lang="ts">
    import { ValueKind } from "$data/api";
    import { automateContext } from "$data/automate/automate";
    import type { Slot, SlotRef } from "$data/automate/node";

    export let slot: Slot;
    export let adaptKind: SlotRef = undefined;

    const { nodes } = automateContext();

    let title = '';

    let kind: ValueKind | "ANY";
    let meta: Record<string, any>;

    $: if (adaptKind && !slot.multiple) {
        const remoteSlot = nodes.getSlot(adaptKind);

        kind = remoteSlot?.kind ?? slot.kind;
        meta = remoteSlot?.meta ?? slot.meta;
    } else {
        kind = slot.kind;
        meta = slot.meta;
    }

    $: if (kind === ValueKind.State) {
        title = `${meta?.possible.join(', ')}`;
    } else if (
        kind === ValueKind.Number && 
        (meta.max !== undefined || meta.min !== undefined)) {
        title = `${meta.min}...${meta.max}`; 
    }
</script>

<div title={title}>
    {slot.label}
    {#if meta?.unit} 
        <span>{meta.unit}</span>
    {/if}
</div>

<style>
    span {
        color: rgba(255,255,255,0.3);
    }
</style>
