import { beforeEach, describe, expect, it, vi } from 'vitest'
import { renderHook } from '@testing-library/react'
import { QueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'
import { useGitOperations } from './useGitOperations'
import { useUIStore } from '@/store/ui-store'
import { defaultPreferences } from '@/types/preferences'

const mockInvoke = vi.fn()
const mockGetAiProviderOverviewQueryOptions = vi.fn()

vi.mock('@/lib/transport', () => ({
  invoke: (...args: unknown[]) => mockInvoke(...args),
}))

vi.mock('@/services/projects', () => ({
  saveWorktreePr: vi.fn(),
  projectsQueryKeys: {
    all: ['projects'],
    worktrees: (projectId: string) => ['projects', 'worktrees', projectId],
  },
}))

vi.mock('@/services/git-status', () => ({
  gitPush: vi.fn(),
  triggerImmediateGitPoll: vi.fn(),
  triggerImmediateRemotePoll: vi.fn(),
  performGitPull: vi.fn(),
}))

vi.mock('@/services/pr-status', () => ({
  prStatusQueryKeys: {
    byWorktree: (worktreeId: string) => ['pr-status', worktreeId],
  },
}))

vi.mock('@/services/ai-provider', () => ({
  getAiProviderOverviewQueryOptions: (...args: unknown[]) =>
    mockGetAiProviderOverviewQueryOptions(...args),
}))

vi.mock('sonner', () => ({
  toast: {
    error: vi.fn(),
    loading: vi.fn(),
    success: vi.fn(),
  },
}))

describe('useGitOperations PR preflight', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    useUIStore.setState({
      preferencesOpen: false,
      preferencesPane: null,
    })
  })

  function renderGitOperations(queryClient: QueryClient) {
    return renderHook(() =>
      useGitOperations({
        activeWorktreeId: 'wt-1',
        activeSessionId: 'session-1',
        activeWorktreePath: '/tmp/worktree',
        worktree: {
          id: 'wt-1',
          project_id: 'project-1',
          name: 'feature-worktree',
          path: '/tmp/worktree',
          branch: 'feature-worktree',
          created_at: 0,
          order: 0,
        },
        project: {
          id: 'project-1',
          name: 'Jean',
          path: '/tmp/project',
          default_branch: 'main',
          added_at: 0,
          order: 0,
        },
        queryClient,
        inputRef: { current: null },
        preferences: defaultPreferences,
      })
    )
  }

  it('opens General settings instead of invoking PR creation when setup is required', async () => {
    const queryClient = new QueryClient({
      defaultOptions: {
        queries: { retry: false },
      },
    })

    mockGetAiProviderOverviewQueryOptions.mockReturnValue({
      queryKey: ['ai-provider-overview', 'wt-1'],
      queryFn: async () => ({
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
            installed: true,
            authenticated: true,
            available: true,
            version: '1.0.0',
            path: '/tmp/opencode',
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
          sessionRecap: 'claude',
          mcp: 'claude',
          customProfiles: 'claude',
          usageReporting: 'claude',
          thinkingControls: 'claude',
          chromeIntegration: 'claude',
        },
      }),
    })

    const { result } = renderGitOperations(queryClient)

    await result.current.handleOpenPr()

    expect(mockInvoke).not.toHaveBeenCalled()
    expect(useUIStore.getState().preferencesPane).toBe('general')
    expect(toast.error).toHaveBeenCalledWith('Codex needs setup', {
      description:
        'PR descriptions are configured to use Codex. Finish install/auth in General settings and try again.',
    })
  })

  it('opens Magic prompts when the selected provider is unsupported', async () => {
    const queryClient = new QueryClient({
      defaultOptions: {
        queries: { retry: false },
      },
    })

    mockGetAiProviderOverviewQueryOptions.mockReturnValue({
      queryKey: ['ai-provider-overview', 'wt-1'],
      queryFn: async () => ({
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
            authenticated: true,
            available: true,
            version: '0.104.0',
            path: '/tmp/codex',
            capabilities: {
              chat: true,
              sessionNaming: true,
              branchNaming: true,
              prContentGeneration: false,
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
            installed: true,
            authenticated: true,
            available: true,
            version: '1.0.0',
            path: '/tmp/opencode',
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
          sessionRecap: 'claude',
          mcp: 'claude',
          customProfiles: 'claude',
          usageReporting: 'claude',
          thinkingControls: 'claude',
          chromeIntegration: 'claude',
        },
      }),
    })

    const { result } = renderGitOperations(queryClient)

    await result.current.handleOpenPr()

    expect(mockInvoke).not.toHaveBeenCalled()
    expect(useUIStore.getState().preferencesPane).toBe('magic-prompts')
    expect(toast.error).toHaveBeenCalledWith(
      'Codex is unsupported for PR content',
      {
        description:
          'Choose a different primary provider for PR Description in Magic prompts.',
      }
    )
  })
})
