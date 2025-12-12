<script lang="ts">
    import {page} from '$app/state';
    import {goto} from '$app/navigation';
    import {onDestroy, onMount} from 'svelte';
    import {
        loadDocumentAtTimestamp,
        createPatch,
        getPatchTimestamps, getDocumentStats, type DocumentStats
    } from '$lib/api';
    import {Button} from '$lib/components/ui/button';
    import {Textarea} from '$lib/components/ui/textarea';
    import {Popover, PopoverContent, PopoverTrigger} from '$lib/components/ui/popover';
    import {Slider} from '$lib/components/ui/slider';
    import {ArrowLeft, Eye, Edit, Clock, ChevronLeft, ChevronRight, ChevronsLeft, ChevronsRight} from '@lucide/svelte';
    import {Markdown} from 'svelte-exmarkdown';
    import rehypeHighlight from 'rehype-highlight';
    import 'highlight.js/styles/github-dark.css';

    const SAVE_THROTTLE_MS = 500;
    const plugins = [
        {
            rehypePlugin: rehypeHighlight,
            options: {detect: true} // auto-detect language
        }
    ];

    let docUuid = $derived(page.params.id!);
    let content = $state('');
    let mode = $state<'write' | 'preview'>('write');
    let timestamps = $state<number[]>([]);
    let timeTravelIndex = $state<number[]>([0]);
    let isTimeTravel = $state(false);
    let timeTravelContent = $state('');
    let stats = $state<DocumentStats | null>(null);
    let saveTimeout: ReturnType<typeof setTimeout> | null = null;
    let isPopoverOpen = $state(false);
    let keyHoldInterval: ReturnType<typeof setInterval> | null = null;
    let keyHoldTimeout: ReturnType<typeof setTimeout> | null = null;

    onMount(async () => {
        await loadTimestamps();
        await loadStats();
        if (timestamps.length > 0) {
            const latest = await loadDocumentAtTimestamp(docUuid, Date.now());
            content = latest;
            timeTravelIndex = [timestamps.length - 1];
        }
    });

    onDestroy(() => {
        if (saveTimeout) clearTimeout(saveTimeout);
        if (keyHoldInterval) clearInterval(keyHoldInterval);
        if (keyHoldTimeout) clearTimeout(keyHoldTimeout);
    });

    async function loadTimestamps() {
        timestamps = await getPatchTimestamps(docUuid);
    }

    async function loadStats() {
        stats = await getDocumentStats(docUuid);
    }

    async function performSave() {
        if (content !== undefined) {
            try {
                await createPatch(docUuid, content);
            } catch (error) {
                // ignore errors for now
            }
            await loadTimestamps();
            await loadStats();
            if (!isTimeTravel) {
                timeTravelIndex = [timestamps.length - 1];
            }
        }
    }

    async function handleContentChange() {
        // THROTTLE: Only schedule if no save is pending
        if (saveTimeout !== null) {
            return; // Save already scheduled for this window
        }

        saveTimeout = setTimeout(async () => {
            await performSave();
            saveTimeout = null; // Clear flag so next keystroke starts a new window
        }, SAVE_THROTTLE_MS);
    }

    async function handleTimeTravelChange(value: number[]) {
        timeTravelIndex = value;
        isTimeTravel = true;
        const timestamp = timestamps[value[0]];
        timeTravelContent = await loadDocumentAtTimestamp(docUuid, timestamp);
    }

    async function jumpToVersion(index: number) {
        if (index < 0 || index >= timestamps.length) return;
        await handleTimeTravelChange([index]);
    }

    async function stepVersion(delta: number) {
        const newIndex = timeTravelIndex[0] + delta;
        if (newIndex >= 0 && newIndex < timestamps.length) {
            await jumpToVersion(newIndex);
        }
    }

    function exitTimeTravel() {
        isTimeTravel = false;
        timeTravelContent = '';
    }

    function formatBytes(bytes: number): string {
        if (bytes === 0) return '0 B';
        const k = 1024;
        const sizes = ['B', 'KB', 'MB'];
        const i = Math.floor(Math.log(bytes) / Math.log(k));
        return Math.round(bytes / Math.pow(k, i) * 100) / 100 + ' ' + sizes[i];
    }

    function formatRelativeTime(timestamp: number): string {
        const now = Date.now();
        const diff = now - timestamp;
        const minutes = Math.floor(diff / 60000);
        const hours = Math.floor(diff / 3600000);
        const days = Math.floor(diff / 86400000);

        if (minutes < 1) return 'just now';
        if (minutes < 60) return `${minutes}m ago`;
        if (hours < 24) return `${hours}h ago`;
        if (days < 7) return `${days}d ago`;
        return new Date(timestamp).toLocaleDateString();
    }

    function handleKeyDown(e: KeyboardEvent) {
        if (!isPopoverOpen) return;

        if (e.key === 'ArrowLeft' || e.key === 'ArrowRight') {
            e.preventDefault();

            // Clear any existing intervals
            if (keyHoldInterval) {
                clearInterval(keyHoldInterval);
                keyHoldInterval = null;
            }
            if (keyHoldTimeout) {
                clearTimeout(keyHoldTimeout);
                keyHoldTimeout = null;
            }

            const direction = e.key === 'ArrowLeft' ? -1 : 1;

            // Immediate first step
            stepVersion(direction);

            // After a short delay, start rapid fire
            keyHoldTimeout = setTimeout(() => {
                keyHoldInterval = setInterval(() => {
                    stepVersion(direction);
                }, 50); // 50ms = 20 steps per second for smooth animation
            }, 300); // 300ms delay before rapid fire starts
        }
    }

    function handleKeyUp(e: KeyboardEvent) {
        if (e.key === 'ArrowLeft' || e.key === 'ArrowRight') {
            if (keyHoldInterval) {
                clearInterval(keyHoldInterval);
                keyHoldInterval = null;
            }
            if (keyHoldTimeout) {
                clearTimeout(keyHoldTimeout);
                keyHoldTimeout = null;
            }
        }
    }

    $effect(() => {
        if (!isTimeTravel && content !== undefined) {
            handleContentChange();
        }
    });

    $effect(() => {
        if (isPopoverOpen) {
            window.addEventListener('keydown', handleKeyDown);
            window.addEventListener('keyup', handleKeyUp);
        } else {
            window.removeEventListener('keydown', handleKeyDown);
            window.removeEventListener('keyup', handleKeyUp);
            if (keyHoldInterval) {
                clearInterval(keyHoldInterval);
                keyHoldInterval = null;
            }
            if (keyHoldTimeout) {
                clearTimeout(keyHoldTimeout);
                keyHoldTimeout = null;
            }
        }

        return () => {
            window.removeEventListener('keydown', handleKeyDown);
            window.removeEventListener('keyup', handleKeyUp);
        };
    });
</script>

<div class="h-screen flex flex-col px-4">
    <header class="border-b p-4 flex items-center justify-between">
        <div class="flex items-center gap-4">
            <Button variant="ghost" size="icon" onclick={() => goto('/')}>
                <ArrowLeft class="h-4 w-4"/>
            </Button>

            {#if stats && stats.total_patches > 0}
                <div class="text-sm text-muted-foreground">
                    <span class="font-mono font-bold text-green-600">
                        {formatBytes(stats.total_delta_bytes)}
                    </span>
                    <span class="mx-2">/</span>
                    <span class="font-mono">
                        {formatBytes(stats.total_uncompressed_bytes)}
                    </span>
                    <span class="ml-2">
                        ({stats.compression_ratio.toFixed(1)}x compression)
                    </span>
                    <span class="ml-2 text-xs">
                        • {stats.total_patches} {stats.total_patches === 1 ? 'patch' : 'patches'}
                    </span>
                    <span class="ml-2 text-xs font-bold text-blue-600">
                        • avg {formatBytes(Math.round(stats.total_delta_bytes / stats.total_patches))}/patch
                    </span>
                </div>
            {/if}
        </div>

        <div class="flex items-center gap-2">
            <Button
                    variant={mode === 'write' ? 'default' : 'outline'}
                    size="sm"
                    onclick={() => mode = 'write'}
            >
                <Edit class="h-4 w-4 mr-2"/>
                Write
            </Button>
            <Button
                    variant={mode === 'preview' ? 'default' : 'outline'}
                    size="sm"
                    onclick={() => mode = 'preview'}
            >
                <Eye class="h-4 w-4 mr-2"/>
                Preview
            </Button>

            {#if timestamps.length > 0}
                <Popover bind:open={isPopoverOpen}>
                    <PopoverTrigger>
                        <Button variant="outline" size="sm">
                            <Clock class="h-4 w-4 mr-2"/>
                            Time Travel ({timestamps.length})
                        </Button>
                    </PopoverTrigger>
                    <PopoverContent class="w-[420px] mr-4">
                        <div class="space-y-4">
                            <div class="flex items-center justify-between">
                                <h4 class="font-medium">Version History</h4>
                                <span class="text-xs text-muted-foreground">
                                    {timeTravelIndex[0] + 1} / {timestamps.length}
                                </span>
                            </div>

                            <div class="text-sm">
                                <div class="font-medium mb-1">
                                    {isTimeTravel
                                        ? new Date(timestamps[timeTravelIndex[0]]).toLocaleString()
                                        : 'Current version'
                                    }
                                </div>
                                {#if isTimeTravel}
                                    <div class="text-xs text-muted-foreground">
                                        {formatRelativeTime(timestamps[timeTravelIndex[0]])}
                                    </div>
                                {/if}
                            </div>

                            <!-- Coarse slider for big jumps -->
                            <div>
                                <label class="text-xs text-muted-foreground mb-2 block">
                                    Quick navigation
                                </label>
                                <Slider
                                        bind:value={timeTravelIndex}
                                        max={timestamps.length - 1}
                                        step={1}
                                        onValueChange={handleTimeTravelChange}
                                />
                            </div>

                            <!-- Fine-grained controls -->
                            <div>
                                <label class="text-xs text-muted-foreground mb-2 block">
                                    Precise control • Use ← → keys
                                </label>
                                <div class="flex items-center gap-2">
                                    <Button
                                            variant="outline"
                                            size="sm"
                                            onclick={() => jumpToVersion(0)}
                                            disabled={timeTravelIndex[0] === 0}
                                            class="flex-1"
                                    >
                                        <ChevronsLeft class="h-3 w-3"/>
                                    </Button>

                                    <Button
                                            variant="outline"
                                            size="sm"
                                            onclick={() => stepVersion(-1)}
                                            disabled={timeTravelIndex[0] === 0}
                                            class="flex-1"
                                    >
                                        <ChevronLeft class="h-3 w-3 mr-1"/>
                                        Prev
                                    </Button>

                                    <Button
                                            variant="outline"
                                            size="sm"
                                            onclick={() => stepVersion(1)}
                                            disabled={timeTravelIndex[0] === timestamps.length - 1}
                                            class="flex-1"
                                    >
                                        Next
                                        <ChevronRight class="h-3 w-3 ml-1"/>
                                    </Button>

                                    <Button
                                            variant="outline"
                                            size="sm"
                                            onclick={() => jumpToVersion(timestamps.length - 1)}
                                            disabled={timeTravelIndex[0] === timestamps.length - 1}
                                            class="flex-1"
                                    >
                                        <ChevronsRight class="h-3 w-3"/>
                                    </Button>
                                </div>
                            </div>

                            {#if isTimeTravel}
                                <Button size="sm" onclick={exitTimeTravel} class="w-full">
                                    Return to Current
                                </Button>
                            {/if}
                        </div>
                    </PopoverContent>
                </Popover>
            {/if}
        </div>
    </header>

    <main class="flex-1 overflow-auto p-6">
        {#if isTimeTravel}
            <div class="max-w-4xl mx-auto">
                <div class="mb-4 p-4 bg-blue-50 dark:bg-blue-950 rounded border border-blue-200">
                    <p class="text-sm font-medium">
                        🕰️ Viewing: {new Date(timestamps[timeTravelIndex[0]]).toLocaleString()}
                    </p>
                </div>
                <div class="prose dark:prose-invert max-w-none">
                    <Markdown md={timeTravelContent} {plugins}/>
                </div>
            </div>
        {:else if mode === 'write'}
            <Textarea
                    bind:value={content}
                    placeholder="Start writing..."
                    class="min-h-[calc(100vh-200px)] font-mono resize-none max-w-4xl mx-auto"
            />
        {:else}
            <div class="max-w-4xl mx-auto prose dark:prose-invert">
                <Markdown md={content} {plugins}/>
            </div>
        {/if}
    </main>
</div>
