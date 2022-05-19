<script lang="ts">
  import Router from 'svelte-spa-router';  
	

  import Icon from "$lib/Icon.svelte";
  import MainMenu from "$lib/MainMenu.svelte";
  import MenuItem from "$lib/MenuItem.svelte";

  import logo from "./assets/logo.svg";

  import routes from './routes';
  import Colors from '$data/colors';
  import { socket } from '$net/pipe';

  let cssColorVariables: string;
  $: cssColorVariables = Object.entries(Colors).map(([k,v]) => `--${k.toLowerCase()}:${v}`).join(';');
</script>

<main style={cssColorVariables}>
  {#if typeof $socket === 'number' && $socket > 500}
  <div class="error">
    ERROR: Could not connect to Rome service
  </div>
  {/if}
  <div class="main2">
    <menubar style="display: none">    
      <div>
        <img width="48" src={logo} alt="Rome"/>      
      </div>

      <div class="name">Rome</div>

      <div class="main-menu">
        <MainMenu>
          <MenuItem url="/" icon="home" />
          <MenuItem url="/devices" icon="microchip" />
          <MenuItem url="/settings" icon="settings-alt" />
        </MainMenu>
      </div>

      <div class="grow"></div>

      <Icon name="log-out" />
    </menubar>

    <section>
      <Router {routes} />
    </section>
  </div>
</main>

<div style=""></div>

<style>
  main {
    min-height: 100%;
    display: flex;
    flex-direction: column;
  }

  .main2 {
    background-color: var(--background);
    display: flex;
    position: relative;
    flex-grow: 1;
  }

  .error {
    padding: 12px;
    text-align: center;
    color: var(--strong);
    font-weight: bold;
    font-size: 16px;

    background-color: var(--error);
    border-bottom: 1px solid #f98383;
    text-shadow: 2px 2px rgba(255,255,255,0.25);
  }

  .name {
    font-size: 11px;
    color: var(--fadedtext);
  }

  section {
    box-sizing: border-box;
    width: 100%;
    padding: 24px;
    margin: 0;
  }

  menubar {
    width: 80px;
    border-right: 2px solid var(--container);

    display: flex;
    flex-direction: column;   
    
    align-items: center;

    padding: 24px 0;
  }

  .main-menu {
    margin-top: 32px;
  }
  
  menubar .grow {
    flex-grow: 1;
  }
</style>
