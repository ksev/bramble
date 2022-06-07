<div class="ledger">
    <slot></slot>
</div>

<style>
    .ledger {
        display: flex;
        padding: 12px;
        border-radius: 8px;

        background-color: var(--container);
        gap: 12px;
    }

    .ledger > :global(div) {
        display: flex;
        justify-content: center;
        align-items: center;
        gap: 5px;    
        font-size: 12px;    
    }
</style>