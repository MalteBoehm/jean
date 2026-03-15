import { beforeEach, describe, expect, it, vi } from 'vitest'
import { renderHook } from '@testing-library/react'
import { useInstalledBackends } from './useInstalledBackends'

const mockUseAiProviderOverview = vi.fn()

vi.mock('@/services/ai-provider', () => ({
  useAiProviderOverview: (options?: { enabled?: boolean }) =>
    mockUseAiProviderOverview(options),
}))

describe('useInstalledBackends', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('maps installed providers from the overview response', () => {
    mockUseAiProviderOverview.mockReturnValue({
      data: {
        providers: {
          claude: { installed: false },
          codex: { installed: true },
          opencode: { installed: true },
        },
      },
      isLoading: false,
    })

    const { result } = renderHook(() => useInstalledBackends())

    expect(result.current.installedBackends).toEqual(['codex', 'opencode'])
    expect(result.current.isLoading).toBe(false)
  })

  it('passes through the enabled flag and loading state', () => {
    mockUseAiProviderOverview.mockReturnValue({
      data: undefined,
      isLoading: true,
    })

    const { result } = renderHook(() =>
      useInstalledBackends({ enabled: false })
    )

    expect(mockUseAiProviderOverview).toHaveBeenCalledWith({ enabled: false })
    expect(result.current.installedBackends).toEqual([])
    expect(result.current.isLoading).toBe(true)
  })
})
