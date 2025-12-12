<script lang="ts">
	import './layout.css';
    import { ModeWatcher } from "mode-watcher";
    import LightSwitch from "$lib/components/LightSwitch.svelte";
	const { children } = $props();
</script>

<ModeWatcher />

<div class="fixed bottom-4 left-4 z-50">
    <LightSwitch />
</div>

<main class="min-h-screen">
    {@render children()}
</main>
