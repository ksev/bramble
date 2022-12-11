<script lang="ts">
    import { automateContext } from "$data/automate/automate";
    import type { Slot, SlotRef } from "$data/automate/node";
    import type { ValueKind } from "$data/device";
    
    const { nodes } = automateContext();

    export let slot: Slot;
    export let adaptKind: SlotRef = null;

    let title = '';

    let kind: ValueKind;

    $: if (adaptKind && !slot.multiple) {
        kind = nodes.getSlot(adaptKind)?.kind ?? slot.kind;
    } else {
        kind = slot.kind;
    }

    $: if (kind.type === 'state') {
        title = `${kind.possible.join(', ')}`;
    }
</script>

<div title={title}>
    {slot.label}
    {#if kind.type === 'number' && kind.unit} 
        <span>{kind.unit}</span>
    {/if}
</div>

<style>
    span {
        color: rgba(255,255,255,0.3);
    }
</style>
