<script lang="ts">
	export let label: string;
	export let value: string = '';
</script>

<div class="form-group">
    <label for="input">{label}</label>
    <input id="input" bind:value={value} />
</div>

<style>
    .form-group {
        display: flex;
        overflow: hidden;
        
        padding: 0;
        gap: 3px;
    }

    label {
        background-color: color-mix(in srgb, var(--section-color) 55%, var(--background));
        padding: 8px;
        display: flex;
        align-items: center;
        border-radius: 2px;
        color: var(--strong);
    }

    input {
        background-color: var(--container);
        border: none;
        outline: none;
        padding: 8px;
        flex-grow: 1;
        color: #fafafa;
        
        border-radius: 2px;
    }
</style>
