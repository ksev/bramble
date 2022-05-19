<menubar>
    <slot></slot>
</menubar>

<style>
    menubar {
        display: flex;
        padding: 6px;
        background-color: var(--container);
        border-radius: 8px;
        gap: 6px;
    }

    menubar > :global(*) {
        background-color: var(--background);
        border-radius: 8px;
        padding: 12px;
        cursor: pointer;

        transition: background-color 100ms linear, color 100ms linear;
    }

    menubar > :global(*:hover) {
        background-color: rgb(35, 35, 57);
    }

    menubar > :global(*:active) {
        background-color: var(--container);
        color: var(--strong);
    }

    menubar > :global(*:disabled) {
        background-color: var(--container);

        cursor: default;
        color: var(--background);
    }
</style>