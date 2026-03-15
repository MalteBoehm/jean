import { describe, expect, it } from 'vitest'
import {
  resolveAiFeatureAvailability,
  type AiProviderOverview,
} from './ai-provider'

const baseOverview: AiProviderOverview = {
  defaultChatProvider: 'claude',
  providers: {
    claude: {
      installed: true,
      authenticated: true,
      available: true,
      version: '1.0.0',
      path: '/tmp/claude',
      capabilities: {
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
      },
    },
    codex: {
      installed: true,
      authenticated: false,
      available: false,
      version: '0.104.0',
      path: '/tmp/codex',
      capabilities: {
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
      },
    },
    opencode: {
      installed: false,
      authenticated: false,
      available: false,
      version: null,
      path: null,
      capabilities: {
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
      },
    },
  },
  selectedFeatureProviders: {
    chat: 'claude',
    investigateIssue: 'claude',
    investigatePr: 'claude',
    investigateWorkflowRun: 'claude',
    investigateSecurityAlert: 'claude',
    investigateAdvisory: 'claude',
    investigateLinearIssue: 'claude',
    prContentGeneration: 'codex',
    commitMessageGeneration: 'claude',
    codeReview: 'claude',
    contextSummarization: 'claude',
    resolveConflicts: 'claude',
    releaseNotes: 'claude',
    sessionNaming: 'claude',
    branchNaming: 'claude',
    sessionRecap: 'opencode',
    mcp: 'claude',
    customProfiles: 'codex',
    usageReporting: 'opencode',
    thinkingControls: 'claude',
    chromeIntegration: 'claude',
  },
}

describe('resolveAiFeatureAvailability', () => {
  it('returns ready when the selected provider is available', () => {
    expect(resolveAiFeatureAvailability(baseOverview, 'chat')).toEqual({
      status: 'ready',
      reason: 'provider_available',
      provider: 'claude',
    })
  })

  it('returns setup_required when the selected provider is not authenticated', () => {
    expect(
      resolveAiFeatureAvailability(baseOverview, 'prContentGeneration')
    ).toEqual({
      status: 'setup_required',
      reason: 'provider_not_authenticated',
      provider: 'codex',
    })
  })

  it('returns setup_required when the selected provider is not installed', () => {
    expect(resolveAiFeatureAvailability(baseOverview, 'sessionRecap')).toEqual({
      status: 'setup_required',
      reason: 'provider_not_installed',
      provider: 'opencode',
    })
  })

  it('returns unsupported when the provider lacks the required capability', () => {
    expect(
      resolveAiFeatureAvailability(baseOverview, 'customProfiles')
    ).toEqual({
      status: 'unsupported',
      reason: 'feature_not_supported',
      provider: 'codex',
    })
  })
})
