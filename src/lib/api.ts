import { invoke } from '@tauri-apps/api/core';

export interface Document {
    uuid: string;
    name: string;
    created_at: number;
}

export interface DocumentStats {
    total_patches: number;
    total_delta_bytes: number;
    total_uncompressed_bytes: number;
    compression_ratio: number;
}

export async function createDocument(name: string): Promise<string> {
    return await invoke('create_document', { name });
}

export async function getDocuments(): Promise<Document[]> {
    return await invoke('get_documents');
}

export async function getDocumentStats(docUuid: string): Promise<DocumentStats> {
    return await invoke('get_document_stats', { docUuid });
}

export async function loadDocumentAtTimestamp(
    docUuid: string,
    timestamp: number
): Promise<string> {
    return await invoke('load_document_at_timestamp', {
        docUuid,
        timestamp
    });
}

export async function createPatch(
    docUuid: string,
    currentContent: string,
    timestamp: number = Date.now()
): Promise<string> {
    return await invoke('create_patch', {
        docUuid,
        currentContent,
        timestamp
    });
}

export async function getPatchTimestamps(docUuid: string): Promise<number[]> {
    return await invoke('get_patch_timestamps', { docUuid });
}