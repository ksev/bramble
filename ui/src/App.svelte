<script lang="ts">
  import Router from 'svelte-spa-router';  
	import routes from './routes';

  import Icon from "./lib/Icon.svelte";
  import MainMenu from "./lib/MainMenu.svelte";
  import MenuItem from "./lib/MenuItem.svelte";

  import logo from "./assets/logo.svg";

  import { colors } from './colors';

  let cssColorVariables: string;
  $: cssColorVariables = Object.entries(colors).map(([k,v]) => `--${k.toLowerCase()}:${v}`).join(';');
</script>

<main style={cssColorVariables}>
  <menubar>    
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
</main>

<div style=""></div>

<style>
  main {
    background-color: var(--background);
    min-height: 100%;
    display: flex;
    position: relative;
  }

  .name {
    font-size: 11px;
    color: var(--fadedtext);
  }

  section {
    box-sizing: border-box;
    width: 100%;
    padding: 24px;
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
