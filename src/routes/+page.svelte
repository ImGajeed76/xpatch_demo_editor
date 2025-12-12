<script lang="ts">
    import { goto } from '$app/navigation';
    import { onMount } from 'svelte';
    import { getDocuments, createDocument, type Document } from '$lib/api';
    import { Button } from '$lib/components/ui/button';
    import { Input } from '$lib/components/ui/input';
    import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';

    let documents = $state<Document[]>([]);
    let newDocName = $state('');
    let loading = $state(true);

    onMount(async () => {
        documents = await getDocuments();
        loading = false;
    });

    async function handleCreate() {
        if (!newDocName.trim()) return;
        const uuid = await createDocument(newDocName);
        newDocName = '';
        goto(`/editor/${uuid}`);
    }
</script>

<div class="container mx-auto p-8 max-w-4xl">
    <h1 class="text-4xl font-bold mb-8">XPatch Demo Editor</h1>

    <Card class="mb-8">
        <CardHeader>
            <CardTitle>Create Document</CardTitle>
        </CardHeader>
        <CardContent>
            <div class="flex gap-2">
                <Input
                        bind:value={newDocName}
                        placeholder="Document name..."
                        onkeydown={(e) => e.key === 'Enter' && handleCreate()}
                />
                <Button onclick={handleCreate}>Create</Button>
            </div>
        </CardContent>
    </Card>

    {#if loading}
        <p class="text-muted-foreground">Loading...</p>
    {:else if documents.length === 0}
        <p class="text-muted-foreground">No documents yet</p>
    {:else}
        <div class="grid gap-4">
            {#each documents as doc}
                <Card
                        class="cursor-pointer hover:bg-accent"
                        onclick={() => goto(`/editor/${doc.uuid}`)}
                >
                    <CardHeader>
                        <CardTitle>{doc.name}</CardTitle>
                    </CardHeader>
                    <CardContent>
                        <p class="text-sm text-muted-foreground">
                            {new Date(doc.created_at).toLocaleString()}
                        </p>
                    </CardContent>
                </Card>
            {/each}
        </div>
    {/if}
</div>