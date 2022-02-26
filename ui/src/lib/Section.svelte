<section>
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
        color: var(--device);
    }

    section > div {
        border-top: 2px solid var(--device);
    }
</style>