<script lang="ts">
    import Icon from "$lib/Icon.svelte";
	import Colors from "$data/colors";
	import Section from "$lib/Section.svelte";
    import SubMenu from "$lib/SubMenu.svelte";
    import TextInput from "$lib/TextInput.svelte";
	import SelectInput from "$lib/SelectInput.svelte";

	import { devices } from "$data/state";
    import api from "$data/api";
    import { ValueKind } from "$data/api_types";
    import WordsInput from "$lib/WordsInput.svelte";

	const types: { value: ValueKind, label: string }[] = [
		{ value: ValueKind.Bool, label: 'Bool' },
		{ value: ValueKind.Number, label: 'Number' },
		{ value: ValueKind.State, label: 'State' },
		{ value: ValueKind.String, label: 'String' }
	];

	let ty: ValueKind;
	let name: string;
	let unit: string;
	let possible: string[];

	let deviceId: string;
	let deviceName: string;
	let working: boolean = false;

	let deviceBufferValid = false;

	function isEmpty(input: string): boolean {
		if (!input) return true;
		return input.trim() === '';
	}

	async function add() {
		try {
			working = true;
			let id = deviceId;

			if (id === 'NEW') {
				const resp = await api.createGenericDevice({ name: deviceName });
				id = resp.genericDevice;
			}

			let meta: Record<string, any> = null;

			if (ty === 'NUMBER') {
				meta = { unit };
			} else if (ty === 'STATE') {
				meta = { possible };
			}

			await api.createValueBuffer({
				deviceId: id,
				name,
				kind: ty,
				meta,
			});

			deviceName = '';
			name = '';
			possible = [];
		} finally {
			working = false;
		}
	}

	$: deviceOptions = [
		{ value: 'NEW', label: 'Create new device...'},
		...$devices.map(d => ({ value: d.id, label: d.name })),
	];

	$: deviceBufferValid = (
		!working &&
		(deviceId !== 'NEW' || deviceId === 'NEW' && !isEmpty(deviceName)) &&		
		!isEmpty(name)
	);
</script>

<div>
	<h1>Virtual</h1>

	<p>
	    Lorem ipsum dolor sit amet, consectetur adipiscing elit. 
		Nam dapibus fermentum nulla nec fringilla. 
		Nulla finibus ligula eu purus consectetur posuere. Etiam ac vulputate libero. 
		In porta elit non ante eleifend, eget convallis libero porta. 
		Nulla facilisi. Nulla sed sem urna. 
		Praesent ipsum nunc, tincidunt eu tristique quis, rutrum ut purus. 
		Ut dapibus porttitor lacinia. 
	</p>

	<Section color={Colors.sourceSink}>
		<div slot="headline">Value Buffer</div>
		<div slot="content">
			<p>
                Lorem ipsum dolor sit amet, consectetur adipiscing elit. 
                Sed purus nibh, <strong>Device Buffer</strong> dictum sit amet urna id, bibendum dapibus metus.
                Pellentesque viverra a est id consequat. 
            </p>

	        <section class="form">
				<div class="meta">
					<SelectInput label="Device" options={deviceOptions} bind:value={deviceId} />
					{#if deviceId === 'NEW'}
						<div style="width: 75%">
							<TextInput label="Device name" bind:value={deviceName} />
						</div>
					{/if}
				</div>				
				<TextInput label="Name" bind:value={name} />
				<div class="meta">
					<SelectInput label="Type of Value" options={types} bind:value={ty} />
					{#if ty === "NUMBER"}
						<div style="width: 75%">
							<TextInput label="Unit" bind:value={unit} />
						</div>
					{/if}
					{#if ty === "STATE"}
						<div style="width: 75%">
							<WordsInput label="Possible" bind:value={possible} />
						</div>
					{/if}
				</div>
	        </section>
        
	        <section class="menu">
	            <SubMenu>
	                <button disabled={!deviceBufferValid} on:click={add}>
	                    Add
	                </button>
	            </SubMenu>
	        </section>
		</div>
	</Section>
	
	<Section color={Colors.device}>
		<div slot="headline">Devices</div>
		<div slot="content">
            <p>
                Lorem ipsum dolor sit amet, consectetur adipiscing elit. 
                Sed purus nibh, <strong>Integration</strong> dictum sit amet urna id, bibendum dapibus metus.
                Pellentesque viverra a est id consequat. 
            </p>

			<div class="devices">
				<SubMenu>
					<div class="integration">
						<span>Button</span>
						<Icon name="xbox-a" size={75} color={Colors.device} />
					</div>

					<div class="integration">
						<span>Switch</span>
						<Icon name="circuit-switch-open" size={75} color={Colors.device} />
					</div>

					<div class="integration">
						<span>Slider</span>
						<Icon name="adjustments-alt" size={75} color={Colors.device} />
					</div>
				</SubMenu>
			</div>
		</div>
	</Section>
</div>

<style>
    .integration {
        display: flex;
        flex-direction: column;
        justify-content: center;
        align-items: center;
        padding: 24px;
        gap: 12px;
        transition: width 500ms linear;
    }

	.devices {
		margin-top: 24px;
		display: flex;
	}

	.form {
		display: flex;
		flex-direction: column;
		margin-top: 24px;
		gap: 6px;
	}

	.meta {
		display: flex;
		gap: 6px;
	}

	.meta > :global(*:first-child) {
		flex-grow: 1;
	}

	.menu {
		display: flex;
		justify-content: flex-end;
	}
</style>