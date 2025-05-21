<script lang="ts">
    import {Textarea} from "$lib/components/ui/textarea/index"; 

    let inputState = $state('');
    const onchange = () => {
        inputState.split('\n')
    }
</script>

<h1>Welcome to the Archivist</h1>

<p>
	<Textarea {onchange} bind:value={inputState} placeholder="Paste deck here..." />
</p>
