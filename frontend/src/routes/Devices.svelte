<script lang="ts">
    import {fade} from 'svelte/transition'

    import Ledger from '$lib/Ledger.svelte';
    import Icon from '$lib/Icon.svelte';
    import Section from '$lib/Section.svelte';
    import SubMenu from '$lib/SubMenu.svelte';

    import Colors from '$data/colors';
</script>

<div in:fade={{duration: 100}}>
    <h1>Device management</h1>

    <p>
        Lorem ipsum dolor sit amet, consectetur adipiscing elit. <strong>Devices</strong>
        Sed elementum metus a augue tempus, id venenatis nibh porttitor. Phasellus id malesuada mauris. 
        Etiam et ante ut <strong>Source</strong> dolor gravida porttitor. 
        Cras molestie porta arcu, eu sollicitudin elit aliquam eu. Nulla non odio placerat, lacinia tortor ac, volutpat nunc. 
        Integer pretium, est eu tempor volutpat <strong>Sink</strong>, tellus ex efficitur enim, vitae sodales ex mi et enim. Nullam id justo lectus. 
        Suspendisse finibus id diam ac volutpat. Morbi vitae sem tellus. <strong>SourceSink</strong> Quisque posuere commodo sagittis. 
        Suspendisse posuere, massa nec tempus porta, eros magna rhoncus purus, ut scelerisque libero urna vitae ante.
    </p>

    <div class="device-menu">
        <SubMenu>
            <a href="#/device/add">Add <strong>Device</strong></a>
        </SubMenu>
      
        <div class="grow"></div>
        
        <Ledger>
            <div>
                <Icon name="microchip" color={Colors.device} size={18} />
                <span>Device</span>
            </div>
            <div>
                <Icon name="activity" color={Colors.feature} size={18} />
                <span>Feature</span>
            </div>
            <div>
                <Icon name="share-alt" color={Colors.automation} size={18} />
                <span>Automation</span>
            </div>
        </Ledger>
    </div>

    <Section color={Colors.device}>
        <span slot="headline">Devices</span>

        <div class="empty" slot="content">
            <span>
                No <strong>Devices</strong> found<br/>
                Add one in the menu above!
            </span>
        </div>
    </Section>
</div>

<style>
    .device-menu {
        display: flex;
        max-width: 1000px;
      
        margin: 24px 0;
        align-items: center;
    }

    .grow {
        flex-grow: 1;
    }

    .empty {
        background-color: var(--container);
        border-radius: 2px;
        margin-top: 3px;
        text-align: center;
        padding: 24px;
        display: flex;
        line-height: 24px;
        justify-content: center;
        align-items: center;
        color: var(--fadedtext);
    }
</style>