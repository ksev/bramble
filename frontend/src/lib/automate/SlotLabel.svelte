<script lang="ts">
  import type { Slot } from "$data/automate/node";

    export let slot: Slot;


    let title = '';

    if (slot.kind.type === 'state') {
        title = `${slot.kind.possible.join(', ')}`;
    }
</script>

<div title={title}>
    {slot.label}
    {#if slot.kind.type === 'number' && slot.kind.unit} 
        <span>{slot.kind.unit}</span>
    {/if}
</div>

<style>
    span {
        color: rgba(255,255,255,0.3);
    }
</style>
