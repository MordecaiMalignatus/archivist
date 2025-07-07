<script lang="ts">
	import * as Select from '$lib/components/ui/select/index';
    import Input from '$lib/components/ui/input/input.svelte';
	import { page } from '$app/state';
	import type { ScryfallCard } from '$lib/scryfallUtils';

	let setCodeValue = $state('');
	let selectedCards: ScryfallCard[] = $state([]); 
	let sets: {name: string, sets: string[]}[] = page.data.boosters; 

    const packSelectionContent = $derived(
       sets.find((s) => s.name === setCodeValue)?.name ?? "Pick A Pack"
    );

</script>

<h1 class="p-4 text-xl font-bold whitespace-normal">Crack-a-pack</h1>

<div id="menu-bar" class="flex flex-nowrap p-4">
	<div class="m-2">
		<Select.Root type="single" bind:value={setCodeValue} on:input={updateCardPool}>
			<Select.Trigger class="w-32]">{packSelectionContent}</Select.Trigger>
			<Select.Content>
				{#each sets as set}
					<Select.Item value={set.name}>{set.name}</Select.Item>
				{/each}
			</Select.Content>
		</Select.Root>
	</div>

	<Input type="text" placeholder="Start typing to find cards..." class="max-w-m m-2" disabled={setCodeValue === ''}/>
 	<button type="submit" class="borders min-w-24 w-30 m-2 rounded-sm bg-sky-300 hover:bg-sky-500">Finish pack</button>
</div>

<div id="current-pack-content">
	{#each selectedCards as card }
		<img src={card.image_uris?.normal} alt={card.name}/>
	{/each}
</div>
