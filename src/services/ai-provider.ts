import { useQuery } from '@tanstack/react-query'
import { hasBackend } from '@/lib/environment'
import { invoke } from '@/lib/transport'
import { logger } from '@/lib/logger'
import type {
  AiProvider,
  AiProviderCapabilities,
  AiProviderOverview,
  AiProviderStatus,
  AiSelectedFeatureProviders,
} from '@/types/ai-provider'

export const aiProviderQueryKeys = {
  all: ['ai-provider-overview'] as const,
  detail: (worktreeId?: string) =>
    [...aiProviderQueryKeys.all, worktreeId ?? 'global'] as const,
}

function createCapabilities(provider: AiProvider): AiProviderCapabilities {
  if (provider === 'claude') {
    return {
      chat: true,
      sessionNaming: true,
      branchNaming: true,
      prContentGeneration: true,
      commitMessageGeneration: true,
      contextSummarization: true,
      sessionRecap: true,
      codeReview: true,
      mcp: true,
      customProfiles: true,
      usageReporting: true,
      thinkingControls: true,
      chromeIntegration: true,
    }
  }

  if (provider === 'codex') {
    return {
      chat: true,
      sessionNaming: true,
      branchNaming: true,
      prContentGeneration: true,
      commitMessageGeneration: true,
      contextSummarization: true,
      sessionRecap: true,
      codeReview: true,
      mcp: true,
      customProfiles: false,
      usageReporting: true,
      thinkingControls: true,
      chromeIntegration: false,
    }
  }

  return {
    chat: true,
    sessionNaming: true,
    branchNaming: true,
    prContentGeneration: true,
    commitMessageGeneration: true,
    contextSummarization: true,
    sessionRecap: true,
    codeReview: true,
    mcp: true,
    customProfiles: false,
    usageReporting: false,
    thinkingControls: true,
    chromeIntegration: false,
  }
}

function createUnavailableStatus(provider: AiProvider): AiProviderStatus {
  return {
    installed: false,
    authenticated: false,
    available: false,
    version: null,
    path: null,
    capabilities: createCapabilities(provider),
  }
}

function createDefaultSelectedFeatureProviders(
  provider: AiProvider
): AiSelectedFeatureProviders {
  return {
    chat: provider,
    investigateIssue: provider,
    investigatePr: provider,
    investigateWorkflowRun: provider,
    investigateSecurityAlert: provider,
    investigateAdvisory: provider,
    investigateLinearIssue: provider,
    prContentGeneration: provider,
    commitMessageGeneration: provider,
    codeReview: provider,
    contextSummarization: provider,
    resolveConflicts: provider,
    releaseNotes: provider,
    sessionNaming: provider,
    branchNaming: provider,
    sessionRecap: provider,
    mcp: provider,
    customProfiles: provider,
    usageReporting: provider,
    thinkingControls: provider,
    chromeIntegration: provider,
  }
}

const DEFAULT_PROVIDER: AiProvider = 'claude'

const DEFAULT_AI_PROVIDER_OVERVIEW: AiProviderOverview = {
  defaultChatProvider: DEFAULT_PROVIDER,
  providers: {
    claude: createUnavailableStatus('claude'),
    codex: createUnavailableStatus('codex'),
    opencode: createUnavailableStatus('opencode'),
  },
  selectedFeatureProviders:
    createDefaultSelectedFeatureProviders(DEFAULT_PROVIDER),
}

export async function fetchAiProviderOverview(
  worktreeId?: string
): Promise<AiProviderOverview> {
  if (!hasBackend()) {
    return DEFAULT_AI_PROVIDER_OVERVIEW
  }

  try {
    return await invoke<AiProviderOverview>('get_ai_provider_overview', {
      worktreeId: worktreeId ?? null,
    })
  } catch (error) {
    logger.error('Failed to load AI provider overview', { error, worktreeId })
    return DEFAULT_AI_PROVIDER_OVERVIEW
  }
}

export function getAiProviderOverviewQueryOptions(options?: {
  worktreeId?: string
}) {
  const worktreeId = options?.worktreeId

  return {
    queryKey: aiProviderQueryKeys.detail(worktreeId),
    queryFn: () => fetchAiProviderOverview(worktreeId),
    staleTime: 1000 * 60 * 5,
    gcTime: 1000 * 60 * 10,
    refetchInterval: 1000 * 60 * 60,
  }
}

export function useAiProviderOverview(options?: {
  worktreeId?: string
  enabled?: boolean
}) {
  return useQuery({
    ...getAiProviderOverviewQueryOptions({ worktreeId: options?.worktreeId }),
    enabled: options?.enabled ?? true,
  })
}
