<script lang="ts">
    export let data: {
        operator: string
    }; 
</script>

<div class="settings">
    <select bind:value={data.operator}>
        <option value="Eq">Equals (=)</option>
        <option value="Lt">Less than (&lt;)</option>
        <option value="Le">Less or equal (&le;)</option>
        <option value="Gt">Greater than (&gt;)</option>
        <option value="Ge">Greater or equal (&ge;)</option>
    </select>
</div>

<style>
    .settings {
        display: flex;
        flex-direction: column;
    }

    select {
        border: 4px solid var(--container);
        background-color: var(--background);
        border-radius: 4px;
        padding: 6px;
        color: var(--strong);
        font-family: 'Roboto', sans-serif;
        color: #cfd2dd;
    }

    select:disabled {
        color: #cfd2dd6e;
    }
</style>