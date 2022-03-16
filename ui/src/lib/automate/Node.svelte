<script lang="ts">    
    export let label: string;
    export let x: number;
    export let y: number;

    let width = 200;
    let height = 130;

    let grabbing = false;

    function mouseDown() {
        grabbing = true;
    }

    function mouseUp() {
        grabbing = false;
    }

    function mouseMove(e: MouseEvent){
        if (!grabbing) return;

        x = Math.ceil((x + e.movementX) / 20) * 20
        y = Math.ceil((y + e.movementY) / 20) * 20
    }
</script>

<div class="node" 
     bind:clientWidth={width} 
     bind:clientHeight={height} 
     style="top: {y}px; left: {x}px">
    <h3 on:mousedown={mouseDown} 
        on:mouseup={mouseUp} 
        on:mousemove={mouseMove} 
        class:grabbing>{label}</h3>
    <div class="node-body">
        <div class="output">
            °C
            <div class="icon numberic"></div>
        </div>
        <div class="output">
            °F
            <div class="icon numberic"></div>
        </div>

        <div class="input">
            <div class="icon numberic"></div>            
            Offset        
            <div class="input-cont">
                <input />
            </div>  
        </div>
    </div>
</div>


<style>
    .node {
        position: absolute;
        width: 200px;        
        filter: drop-shadow(0px 0px 4px rgba(0,0,0,0.7));
    }

    h3 {
        border-radius: 4px 4px 0 0;
        cursor: grab;
        background-color: var(--sink);
        color: var(--strong);        
        padding: 8px;
        font-size: 14px;
        font-weight: normal;
        cursor: grab;
        filter: drop-shadow(0px 0px 1px rgba(255,255,255,0.2));
    }

    h3.grabbing {
        cursor: grabbing;
    }

    .node-body {
        background-color: var(--background);
        display: flex;
        padding: 8px 0;
        gap: 8px;
        flex-direction: column;

        border-radius: 0 0 4px 4px;
    }  

    .output {
        height: 20px;
        display: flex;
        justify-content: right;
        align-items: center;
        gap: 8px;

        margin-right: -5px;
    }

    .input {
        height: 20px;
        display: flex;
        justify-content: left;
        align-items: center;
        gap: 8px;

        margin-left: -5px;
    }

    .input .input-cont {
        padding-right: 8px;
        flex-shrink: 1;
    }

    .input input {
        border: none;
        width: 100%;
        height: 100%;
        margin: 0;
        background-color: var(--container);
        border-radius: 4px;
        font-size: 12px;
    }

    .icon {
        background-color: var(--device);
        width: 12px;
        height: 12px;
        min-width: 12px;
        min-height: 12px;
        border: 1px solid #000;
    }

    .icon:hover {
        filter:saturate(5);
    }

    .icon.numberic {
        border-radius: 6px;
    }
</style>