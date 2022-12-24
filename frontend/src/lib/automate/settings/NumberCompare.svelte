<div class="settings">
    <select>
        <option selected>Equals (=)</option>
        <option>Less than (&lt;)</option>
        <option>Greater than (&gt;)</option>
    </select>
</div>

<style>
    .settings {
        display: flex;
        flex-direction: column;
    }

    select {
        border: 4px solid var(--container);
        background-color: var(--background);
        border-radius: 4px;
        padding: 6px;
        color: var(--strong);
        font-family: 'Roboto', sans-serif;
        color: #cfd2dd;
    }

    select:disabled {
        color: #cfd2dd6e;
    }
</style>