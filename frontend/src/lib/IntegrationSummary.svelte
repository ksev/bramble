<script lang="ts">
    import { ValueDirection } from "$data/api";
    import { devices } from "$data/devices";
    import { flatMap } from "$data/iterators";
    import Icon from "./Icon.svelte";
	import Colors from "$data/colors";

	export let name: string;

    let devs = 0;
    let sources = 0;
    let sinks = 0;
    
    let children = flatMap(devices.byIntegration(name), d => devices.children(d.id));

    for (const d of children) {
        devs += 1;

        for (const f of d.features) {
            switch (f.direction) {
                case ValueDirection.Sink:
                    sinks += 1;
                    break;
                case ValueDirection.Source:
                    sources += 1;
                    break;
                case ValueDirection.SourceSink:
                    sinks += 1;
                    sources += 1;
            }
        }
    }
</script>

{#if devs}
	<div class="integration-summary">
	    <div class="stat">
	        <Icon name="cpu" size={16} color={Colors.device} />
	        <strong>{devs}</strong> 
	        Devices
	    </div>
	    <div class="stat">
	        <Icon name="stack-push" size={16} color={Colors.source} />
	        <strong>{sinks}</strong> 
	        Inputs
	    </div>
	    <div class="stat">
	        <Icon name="stack-pop" size={16} color={Colors.sink} />
	        <strong>{sources}</strong> 
	        Outputs
	    </div>
	</div>
{/if}

<style>
	
    .integration-summary {
        display: flex;
        flex-direction: column; 
        gap: 2px;
        flex-grow: 1;
    }

    .stat {
        display: flex;
        gap: 6px;
        align-items: center;
    }
</style>
