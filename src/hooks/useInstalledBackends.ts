import { useMemo } from 'react'
import { useAiProviderOverview } from '@/services/ai-provider'
import type { CliBackend } from '@/types/preferences'

/**
 * Returns only the backends whose CLIs are currently installed.
 * Use this to filter backend selection UI so users can't pick uninstalled ones.
 */
export function useInstalledBackends(options?: { enabled?: boolean }) {
  const enabled = options?.enabled ?? true
  const overview = useAiProviderOverview({ enabled })

  const installedBackends = useMemo(() => {
    const backends: CliBackend[] = []
    if (overview.data?.providers.claude.installed) backends.push('claude')
    if (overview.data?.providers.codex.installed) backends.push('codex')
    if (overview.data?.providers.opencode.installed) backends.push('opencode')
    return backends
  }, [
    overview.data?.providers.claude.installed,
    overview.data?.providers.codex.installed,
    overview.data?.providers.opencode.installed,
  ])

  return {
    installedBackends,
    isLoading: overview.isLoading,
  }
}
