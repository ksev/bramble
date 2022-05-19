<script lang="ts">
    import type { Color } from '$data/colors';

    export let color: Color | string;
</script>

<section style="--section-color: {color.toString()}">
    <h2><slot name="headline"></slot></h2>

    <div>
        <slot name="content"></slot>
    </div>
</section>

<style>
    section {  
        max-width: 1000px;       
        margin-top: 24px;
    }

    section h2 {
        color: var(--section-color);
    }

    section > div {
        border-top: 2px solid var(--section-color);
    }
</style>