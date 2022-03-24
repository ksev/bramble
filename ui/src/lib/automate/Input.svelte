<script lang="ts">
    import { Input, IOId } from "./automate";
    import IOAnchor from "./IOAnchor.svelte";

    export let nodeId: number;
    export let data: Input;

    const ioid = new IOId(nodeId, data.id);
</script>

<div class="input">
    <IOAnchor id={ioid} type={data.type} direction={"input"} />
    {data.label}
    <!--
    <div class="input-cont">
        <input />
    </div>  
    -->
</div>

<style>   
    .input {
        height: 20px;
        display: flex;
        justify-content: left;
        align-items: center;
        gap: 8px;

        margin-left: -6px;
    }

    /*
    .input .input-cont {
        padding-right: 8px;
        flex-shrink: 1;
    }

    .input input {
        border: none;
        width: 100%;
        height: 100%;
        margin: 0;
        padding: 4px;
        background-color: var(--container);
        border-radius: 4px;
        font-size: 12px;
    }
    */
</style>
