<script lang="ts">
  import Router from "svelte-spa-router";
  import { apiConnection } from "$data/api";
  import { current } from "$data/notification";

  import Icon from "$lib/Icon.svelte";
  import MainMenu from "$lib/MainMenu.svelte";
  import MenuItem from "$lib/MenuItem.svelte";

  import logo from "./assets/logo.svg";

  import routes from "./routes";
  import Colors from "$data/colors";
  import { fade } from "svelte/transition";

  let cssColorVariables: string;
  $: cssColorVariables = Object.entries(Colors)
    .map(([k, v]) => `--${k.toLowerCase()}:${v}`)
    .join(";");

  $: notif =
    typeof $apiConnection === "number" && $apiConnection > 500
      ? {
          message: "Cannot connect to Bramble service",
          type: "error",
        }
      : $current;
</script>

<main style={cssColorVariables}>
  {#if notif}
    <div class="notification {notif.type}" transition:fade|local>
      {notif.message}
    </div>
  {/if}
  <menubar style="display: none">
    <div>
      <img width="48" src={logo} alt="Bramble" />
    </div>

    <div class="name">Bramble</div>

    <div class="main-menu">
      <MainMenu>
        <MenuItem url="/" icon="home" />
        <MenuItem url="/devices" icon="microchip" />
        <MenuItem url="/settings" icon="settings-alt" />
      </MainMenu>
    </div>

    <div class="grow" />

    <Icon name="log-out" />
  </menubar>

  <section>
    <Router {routes} />
  </section>
</main>

<style>
  main {
    min-height: 100%;
    display: flex;
    background-color: var(--background);
    position: relative;
  }

  .name {
    font-size: 11px;
    color: var(--fadedtext);
  }

  section {
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

  .notification {
    padding: 12px;
    text-align: center;
    color: var(--strong);
    font-weight: bold;
    font-size: 16px;
    position: fixed;
    width: 100%;
    z-index: 999;
    text-shadow: 2px 2px rgba(255, 255, 255, 0.25);

    transition: 500ms border, 500ms background-color;
  }

  .notification.error {
    background-color: var(--error);
    border-bottom: 1px solid #f98383;
  }

  .notification.notify {
    background-color: var(--container);
    border-bottom: 1px solid rgba(255, 255, 255, 0.25);
  }

  .notification.success {
    border-bottom: 1px solid rgba(255, 255, 255, 0.25);
    background-color: var(--success);
  }
</style>
