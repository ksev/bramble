<script lang="ts">
    import Colors, { Color } from '$data/colors'
    
    export let name: string;
    export let color: Color | string = Colors.icon;
    export let size = 24;
</script>

<div style="background: {color}; mask: url(/icons/{name}.svg) 0 0/{size}px {size}px; width: {size}px; height:{size}px; -webkit-mask: url(/icons/{name}.svg) 0 0/{size}px {size}px; width: {size}px; height:{size}px"></div>

<style>
    div {
        transition: background-color 200ms linear;
    }
</style>
