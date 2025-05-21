<script lang="ts">
	import * as Select from '$lib/components/ui/select/index';
	let value = $state('pick a set');
	const sets = [
		{ value: 'DSK', label: 'Duskmourn (DSK)' },
		{ value: 'BLB', label: 'Bloomburrow (BLB)' },
		{ value: 'TDM', label: 'Tarkir Dragonstorm (TDM)' }
	];
    const triggerContent = $derived(
        sets.find((s) => s.value === value)?.label ?? "Select Set"
    );
</script>

<h1 class="p-8 text-xl font-bold whitespace-normal">Crack-a-pack</h1>

<div id="menu-bar" class="flex flex-nowrap">
	<div id="set-selection" class="m-4">
		<Select.Root type="single" bind:value>
			<Select.Trigger class="w-[180px]">{triggerContent}</Select.Trigger>
			<Select.Content>
				{#each sets as set}
					<Select.Item value={set.value} label={set.label}>{set.label}</Select.Item>
				{/each}
			</Select.Content>
		</Select.Root>
	</div>

	<input type="text" class="borders p-2 rounded-sm"/>
	<button type="submit" class="borders p-4 rounded-sm bg-sky-300">Finish pack</button>
</div>

<div id="current-pack-content"></div>
