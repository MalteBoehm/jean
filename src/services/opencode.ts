import { useQuery } from '@tanstack/react-query'
import { invoke } from '@/lib/transport'
import { hasBackend } from '@/lib/environment'

const isTauri = hasBackend

export interface OpenCodeCliStatus {
    installed: boolean
    version: string | null
}

export interface OpenCodeModel {
    id: string
    name: string
    provider: string
}

/** Check if OpenCode CLI is installed */
export function useOpenCodeInstalled() {
    return useQuery({
        queryKey: ['opencode', 'installed'],
        queryFn: async (): Promise<OpenCodeCliStatus> => {
            if (!isTauri()) {
                return { installed: false, version: null }
            }
            try {
                return await invoke<OpenCodeCliStatus>('check_opencode_installed')
            } catch {
                return { installed: false, version: null }
            }
        },
        staleTime: 1000 * 60 * 5, // Cache for 5 min
        retry: false,
    })
}

/** List available OpenCode models */
export function useOpenCodeModels(enabled = true) {
    return useQuery({
        queryKey: ['opencode', 'models'],
        queryFn: async (): Promise<OpenCodeModel[]> => {
            if (!isTauri()) {
                return []
            }
            try {
                return await invoke<OpenCodeModel[]>('list_opencode_models')
            } catch {
                return []
            }
        },
        staleTime: 1000 * 60 * 5,
        enabled,
        retry: false,
    })
}
