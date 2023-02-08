<script lang="ts">
    import Icon from "$lib/Icon.svelte";
    import { automateContext } from "$data/automate/automate";
    import { ROOT, type Action } from "$data/automate/contextMenu";

    const ctx = automateContext();
    const { contextMenu } = ctx;

    let timer: number;
    let items = ROOT;

    function onMouseLeave() {
       timer = setTimeout(() => contextMenu.set(null), 350);
    }
    
    function onMouseEnter() {
        clearTimeout(timer); 
    }

    function doAction(action: Action) {
        if (action.type === 'load') {
            action.fn(ctx);
            contextMenu.set(null);
            return;
        }

        items = action.fn();
    }
</script>


<div class="sensor" 
    style="top: {$contextMenu.y-34}px; left: {$contextMenu.x-34}px" 
    on:mouseenter={onMouseEnter} 
    on:mouseleave={onMouseLeave}>
    <div class="menu">
        <div class="search">
            <input placeholder="Search" />
        </div>

        <div class="items" on:wheel={e => e.stopPropagation()}>
            {#if items.length === 0}
                <div class="empty"><h3>Empty</h3></div>
            {/if}
            {#each items as item}
            <div class="menu-item">
                <button on:click={() => doAction(item.action)}>
                    <div class="label">
                        <Icon name={item.icon} size={16} /> {item.text} 
                    </div>
                    {#if item.category}
                        <Icon name="chevron-right" size={16} />
                    {/if}
                </button>
            </div>
            {/each}
        </div>
    </div>
</div>

<style>
    .sensor {
        position: absolute;
        padding: 10px;
        z-index: 10000;
    }

    .menu {
        background-color: var(--background);
        display: flex;
        padding: 9px;
        border-radius: 8px;
        width: 220px;
        flex-direction: column;

        transition: 200ms width;
        filter: drop-shadow(0px 0px 4px rgba(0,0,0,0.4));
    }

    .items {
        overflow-y: auto;
        max-height: 235px;
        scrollbar-width: none;
    }

    .menu-item {
        background-color: var(--container);
        padding: 2px 2px 0 2px;
        border-radius: 8px;
    }

    .menu-item:last-child {
        padding-bottom: 2px;
    }
    
    .menu-item > * {
        background-color: var(--background);
        padding: 6px;
        border-radius: 8px;
        display: flex;
        justify-content: space-between;
        align-items: center;
        cursor: pointer;
        width: 100%;
    }

    .menu-item:hover > *:not(.search-item) {
        background-color: rgb(35, 35, 57);
    }

    .label {
        display: flex;
        align-items: center;
        gap: 6px;
    }

    .search {
        background-color: var(--container);
        border-radius: 8px;
        margin-bottom: 9px;
        padding: 6px;
        display: flex;
        flex-direction: column;
        align-items: center;
    }

    .search input {
        border: none;
        background:none;
        display: block;

        outline: none;
        font-size: 14px;
    }

    .empty {
        display: flex;
        justify-content: center;
        align-items: center;
        padding: 4px;
        border-radius: 8px;
        flex-direction: column;
    }
</style>