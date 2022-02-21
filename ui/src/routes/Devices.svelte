<script lang="ts">
    import {fade} from 'svelte/transition'

    import Icon from '../lib/Icon.svelte';
    import Source from '../lib/Source.svelte';
    import Colors from '../colors';
</script>

<div in:fade={{duration: 100}}>
    <h1>Device management</h1>

    <p>
        Lorem ipsum dolor sit amet, consectetur adipiscing elit. 
        Sed elementum metus a augue tempus, id venenatis nibh porttitor. Phasellus id malesuada mauris. 
        Etiam et ante ut <strong>Source</strong> dolor gravida porttitor. 
        Cras molestie porta arcu, eu sollicitudin elit aliquam eu. Nulla non odio placerat, lacinia tortor ac, volutpat nunc. 
        Integer pretium, est eu tempor volutpat <strong>Sink</strong>, tellus ex efficitur enim, vitae sodales ex mi et enim. Nullam id justo lectus. 
        Suspendisse finibus id diam ac volutpat. Morbi vitae sem tellus. <strong>SourceSink</strong> Quisque posuere commodo sagittis. 
        Suspendisse posuere, massa nec tempus porta, eros magna rhoncus purus, ut scelerisque libero urna vitae ante.
    </p>

    <div class="sinks">
        <h2>Sinks</h2>

        <div>
            <div class="empty">
                <span>
                    No <strong>sinks</strong> found
                </span>
            </div>
        </div>
    </div>

    <div class="sources">
        <h2>Sources</h2>

        <div>
            <Source name="sensor.bed_tmp_temperature" sensor="source" />
            <Source name="sensor.liv_brk_computer" sensor="sourcesink" />
            <Source name="sensor.liv_tmp_temperature" sensor="source" />
            <Source name="sensor.out_tmp_temperature" sensor="source" />
        </div>
    </div>

    <div class="ledger">
        <div>
            <Icon name="microchip" color={Colors.source} size={18} />
            <span>Source</span>
        </div>
        <div>
            <Icon name="microchip" color={Colors.sink} size={18} />
            <span>Sink</span>
        </div>
        <div>
            <Icon name="microchip" color={Colors.sourcesink} size={18} />
            <span>SourceSink</span>
        </div>
    </div>
</div>

<style>
    .ledger {
        display: inline-flex;
        padding: 12px;
        border-radius: 8px;

        background-color: var(--container);
        gap: 12px;
        margin: 24px 0;
    }

    .ledger > div {
        display: flex;
        justify-content: center;
        align-items: center;
        gap: 5px;    
        font-size: 12px;    
    }

    h1 {
        font-size: 30px;  
        color: var(--strong);      
    }
    
    strong {
        color: var(--strong);
    }

    p {
        max-width: 700px;
        padding: 0;
        margin: 0;
        margin-top: 12px;
        font-size: 14px;

        color: var(--fadedtext);
    }

    .sinks {
        margin-top: 24px;     
        max-width: 1000px;       
    }

    .sinks h2 {
        color: var(--sink);
    }

    .sinks > div {
        border-top: 2px solid var(--sink);
    }

    .sources {
        margin-top: 24px;
        max-width: 1000px;        
    }

    .sources > div {
        border-top: 2px solid var(--source);
    }

    .sources h2 {
        color: var(--source);
    }


    .empty {
        background-color: var(--container);
        margin-top: 3px;
        box-sizing: border-box;
        padding: 24px;
        display: flex;
        justify-content: center;
        align-items: center;
        color: var(--fadedtext);
    }
</style>