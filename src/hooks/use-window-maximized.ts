import { useEffect, useState } from 'react'
import { getCurrentWindow } from '@tauri-apps/api/window'

/**
 * Hook to track whether the current window is maximized.
 * Useful for adjusting UI elements like border radius when maximized.
 */
export function useWindowMaximized() {
  const [isMaximized, setIsMaximized] = useState(false)

  useEffect(() => {
    const checkMaximized = async () => {
      try {
        const appWindow = getCurrentWindow()
        const maximized = await appWindow.isMaximized()
        setIsMaximized(maximized)
      } catch {
        // Ignore errors
      }
    }

    checkMaximized()

    // Listen for resize events to update maximized state
    const setupResizeListener = async () => {
      try {
        const appWindow = getCurrentWindow()
        const unlisten = await appWindow.onResized(async () => {
          const maximized = await appWindow.isMaximized()
          setIsMaximized(maximized)
        })
        return unlisten
      } catch {
        return null
      }
    }

    let unlisten: (() => void) | null = null
    setupResizeListener().then(fn => {
      unlisten = fn
    })

    return () => {
      if (unlisten) {
        unlisten()
      }
    }
  }, [])

  return isMaximized
}
