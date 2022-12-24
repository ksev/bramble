<script lang="ts">
	type T = $$Generic;

	export let label: string;
	export let options: {value: T, label: string}[] = [];
	export let value: T = undefined;

	let idx = 0;

	$: value = options[idx]?.value
</script>

<div class="form-group">
    <label for="input">{label}</label>
	<div class="select-wrap">
	    <select id="input" bind:value={idx}>
			{#each options as option, i}
				<option value={i}>{option.label}</option>
			{/each}
		</select>
	</div>
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
        white-space: nowrap;
    }

    select {
        border: none;
        outline: none;
        flex-grow: 1;
        color: #fafafa;        
    }

	.select-wrap {
        background-color: var(--container);
		padding: 6px;
		box-sizing: content-box;
		display: flex;	
		width: 100%;
        border-radius: 2px;
	}

    .select-wrap:hover {
        background-color: var(--containerhigh);
    }
</style>
