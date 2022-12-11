<script lang="ts">
    export let possible: string[]; 
</script>

<div class="settings">
    {#if possible.length === 0} 
        <select disabled>
            <option>Connect Input to enable</option>
        </select>
    {:else}
        <select>
            {#each possible as state}
                <option>{state}</option>
            {/each}
        </select>
    {/if}    
</div>

<style>
    .settings {
        display: flex;
        flex-direction: column;
        /* Make sure we stay in the default size, it's 200 - padding */
        width: 176px;
    }
</style>