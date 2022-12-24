<script lang="ts">
	export let label: string;
    export let value: string[] = [];

    let textValue = '';
    const rexp = /\s/;

    function blur() {
        if (!textValue) return;

        value = [
            ...value,
            textValue,
        ];

        textValue = '';
    }

    function remove(word: string) {
        value = value.filter(v => v !== word);
    }

    $: {
        const split = textValue.split(rexp);

        value = [
            ...value,
            ...split.slice(0, -1)
        ];

        textValue = split.slice(-1)[0];
    }
</script>

<div class="form-group">
    <label for="input">{label}</label>
    <div class="words">
        {#each value as word}
            <div on:click={() => remove(word)}>{word}</div>
        {/each}
        <input id="input" bind:value={textValue} on:blur={blur} />
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
    }

    input {        
        border: none;
        outline: none;
        padding: 8px;
        flex-grow: 1;
        color: #fafafa;            
    }

    .words {
        display: flex;
        gap: 3px;
        flex-grow: 1;
    }
    
    .words > * {
        background-color: var(--container);
        border-radius: 2px;
    }

    .words > div {
        display: flex;
        justify-content: center;
        align-items: center;
        padding: 0 8px;
    }

    .words > div:hover {
        background-color: var(--containerhigh);
        cursor: pointer;
    }
</style>
