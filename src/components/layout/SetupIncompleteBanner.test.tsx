import { beforeEach, describe, expect, it, vi } from 'vitest'
import { fireEvent, render, screen } from '@testing-library/react'
import { SetupIncompleteBanner } from './SetupIncompleteBanner'
import { useUIStore } from '@/store/ui-store'

const mockUseAiProviderOverview = vi.fn()
const mockUseGhCliStatus = vi.fn()
const mockUseGhCliAuth = vi.fn()

vi.mock('@/services/ai-provider', () => ({
  useAiProviderOverview: () => mockUseAiProviderOverview(),
}))

vi.mock('@/services/gh-cli', () => ({
  useGhCliStatus: () => mockUseGhCliStatus(),
  useGhCliAuth: () => mockUseGhCliAuth(),
}))

vi.mock('@/lib/environment', () => ({
  isNativeApp: () => true,
}))

describe('SetupIncompleteBanner', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    useUIStore.setState({
      onboardingDismissed: true,
      onboardingOpen: false,
      onboardingManuallyTriggered: false,
    })
  })

  it('stays hidden when GitHub and at least one chat provider are ready', () => {
    mockUseAiProviderOverview.mockReturnValue({
      isLoading: false,
      data: {
        providers: {
          claude: {
            available: false,
            capabilities: { chat: true },
          },
          codex: {
            available: true,
            capabilities: { chat: true },
          },
          opencode: {
            available: false,
            capabilities: { chat: true },
          },
        },
      },
    })
    mockUseGhCliStatus.mockReturnValue({
      isLoading: false,
      data: { installed: true },
    })
    mockUseGhCliAuth.mockReturnValue({
      isLoading: false,
      data: { authenticated: true },
    })

    const { container } = render(<SetupIncompleteBanner />)

    expect(container).toBeEmptyDOMElement()
  })

  it('opens onboarding again when setup is incomplete', () => {
    mockUseAiProviderOverview.mockReturnValue({
      isLoading: false,
      data: {
        providers: {
          claude: {
            available: false,
            capabilities: { chat: true },
          },
          codex: {
            available: false,
            capabilities: { chat: true },
          },
          opencode: {
            available: false,
            capabilities: { chat: true },
          },
        },
      },
    })
    mockUseGhCliStatus.mockReturnValue({
      isLoading: false,
      data: { installed: true },
    })
    mockUseGhCliAuth.mockReturnValue({
      isLoading: false,
      data: { authenticated: false },
    })

    render(<SetupIncompleteBanner />)

    expect(screen.getByText(/setup incomplete/i)).toBeInTheDocument()

    fireEvent.click(screen.getByRole('button', { name: /complete setup/i }))

    expect(useUIStore.getState().onboardingOpen).toBe(true)
    expect(useUIStore.getState().onboardingDismissed).toBe(false)
    expect(useUIStore.getState().onboardingManuallyTriggered).toBe(true)
  })
})
