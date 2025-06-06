<script lang="ts">
	import * as Select from '$lib/components/ui/select/index';
    import Input from '$lib/components/ui/input/input.svelte';

	let value = $state('pick a set');
	let sets = [];
    const triggerContent = $derived(
       sets.find((s) => s.value === value)?.label ?? "Select Set"
    );
</script>

<h1 class="p-8 text-xl font-bold whitespace-normal">Crack-a-pack</h1>

<div id="menu-bar" class="flex flex-nowrap">
	<div class="m-2">
		<Select.Root type="single" bind:value>
			<Select.Trigger class="w-32]">{triggerContent}</Select.Trigger>
			<Select.Content>
				{#each sets as set}
					<Select.Item value={set.value}>{set.label}</Select.Item>
				{/each}
			</Select.Content>
		</Select.Root>
	</div>

	<Input type="text" placeholder="Start typing to find cards..." class="max-w-m m-2" />
 	<button type="submit" class="borders min-w-24 w-30 m-2 rounded-sm bg-sky-300 hover:bg-sky-500">Finish pack</button>
</div>

<div id="current-pack-content"></div>
