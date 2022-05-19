<script lang="ts">
    import {fade} from 'svelte/transition'

    document.location.href = '/#/devices';
</script>

<div in:fade={{duration: 100}}>
    <h1>Dashboard</h1>
</div>
