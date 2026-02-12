import { useQuery } from '@tanstack/react-query'
import { invoke } from '@/lib/transport'

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
        queryFn: () => invoke<OpenCodeCliStatus>('check_opencode_installed'),
        staleTime: 1000 * 60 * 5, // Cache for 5 min
        retry: false,
    })
}

/** List available OpenCode models */
export function useOpenCodeModels(enabled = true) {
    return useQuery({
        queryKey: ['opencode', 'models'],
        queryFn: () => invoke<OpenCodeModel[]>('list_opencode_models'),
        staleTime: 1000 * 60 * 5,
        enabled,
        retry: false,
    })
}
