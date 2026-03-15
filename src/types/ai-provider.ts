export type AiProvider = 'claude' | 'codex' | 'opencode'

export interface AiProviderCapabilities {
  chat: boolean
  sessionNaming: boolean
  branchNaming: boolean
  prContentGeneration: boolean
  commitMessageGeneration: boolean
  contextSummarization: boolean
  sessionRecap: boolean
  codeReview: boolean
  mcp: boolean
  customProfiles: boolean
  usageReporting: boolean
  thinkingControls: boolean
  chromeIntegration: boolean
}

export interface AiProviderStatus {
  installed: boolean
  authenticated: boolean
  available: boolean
  version: string | null
  path: string | null
  capabilities: AiProviderCapabilities
}

export interface AiSelectedFeatureProviders {
  chat: AiProvider
  investigateIssue: AiProvider
  investigatePr: AiProvider
  investigateWorkflowRun: AiProvider
  investigateSecurityAlert: AiProvider
  investigateAdvisory: AiProvider
  investigateLinearIssue: AiProvider
  prContentGeneration: AiProvider
  commitMessageGeneration: AiProvider
  codeReview: AiProvider
  contextSummarization: AiProvider
  resolveConflicts: AiProvider
  releaseNotes: AiProvider
  sessionNaming: AiProvider
  branchNaming: AiProvider
  sessionRecap: AiProvider
  mcp: AiProvider
  customProfiles: AiProvider
  usageReporting: AiProvider
  thinkingControls: AiProvider
  chromeIntegration: AiProvider
}

export interface AiProviderOverview {
  defaultChatProvider: AiProvider
  providers: Record<AiProvider, AiProviderStatus>
  selectedFeatureProviders: AiSelectedFeatureProviders
}

export type AiFeature = keyof AiSelectedFeatureProviders
export type AiCapabilityFeature = keyof AiProviderCapabilities

export type AiFeatureAvailabilityState =
  | 'ready'
  | 'setup_required'
  | 'unsupported'

export type AiFeatureAvailabilityReason =
  | 'provider_available'
  | 'provider_not_installed'
  | 'provider_not_authenticated'
  | 'provider_not_available'
  | 'feature_not_supported'

export interface AiFeatureAvailability {
  status: AiFeatureAvailabilityState
  reason: AiFeatureAvailabilityReason
  provider: AiProvider
}

export const AI_PROVIDER_LABELS: Record<AiProvider, string> = {
  claude: 'Claude',
  codex: 'Codex',
  opencode: 'OpenCode',
}

const AI_FEATURE_CAPABILITY_MAP: Record<AiFeature, AiCapabilityFeature> = {
  chat: 'chat',
  investigateIssue: 'chat',
  investigatePr: 'chat',
  investigateWorkflowRun: 'chat',
  investigateSecurityAlert: 'chat',
  investigateAdvisory: 'chat',
  investigateLinearIssue: 'chat',
  prContentGeneration: 'prContentGeneration',
  commitMessageGeneration: 'commitMessageGeneration',
  codeReview: 'codeReview',
  contextSummarization: 'contextSummarization',
  resolveConflicts: 'chat',
  releaseNotes: 'chat',
  sessionNaming: 'sessionNaming',
  branchNaming: 'branchNaming',
  sessionRecap: 'sessionRecap',
  mcp: 'mcp',
  customProfiles: 'customProfiles',
  usageReporting: 'usageReporting',
  thinkingControls: 'thinkingControls',
  chromeIntegration: 'chromeIntegration',
}

export function resolveAiFeatureAvailability(
  overview: AiProviderOverview,
  feature: AiFeature
): AiFeatureAvailability {
  const provider = overview.selectedFeatureProviders[feature]
  const providerStatus = overview.providers[provider]
  const capability = AI_FEATURE_CAPABILITY_MAP[feature]

  if (!providerStatus.capabilities[capability]) {
    return {
      status: 'unsupported',
      reason: 'feature_not_supported',
      provider,
    }
  }

  if (providerStatus.available) {
    return {
      status: 'ready',
      reason: 'provider_available',
      provider,
    }
  }

  if (!providerStatus.installed) {
    return {
      status: 'setup_required',
      reason: 'provider_not_installed',
      provider,
    }
  }

  if (!providerStatus.authenticated) {
    return {
      status: 'setup_required',
      reason: 'provider_not_authenticated',
      provider,
    }
  }

  return {
    status: 'setup_required',
    reason: 'provider_not_available',
    provider,
  }
}
