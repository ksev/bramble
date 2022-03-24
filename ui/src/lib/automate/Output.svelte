<script lang="ts">
    import { IOId, Output } from "./automate";
    import IOAnchor from "./IOAnchor.svelte";

    export let nodeId: number;
    export let data: Output;

    const ioid = new IOId(nodeId, data.id);
</script>

<div class="output">
    {data.label}
    <IOAnchor id={ioid} type={data.type} direction={"output"} />
</div>

<style>
     .output {
        height: 20px;
        display: flex;
        justify-content: right;
        align-items: center;
        gap: 8px;

        margin-right: -6px;
    }    
</style>