import { useState, useEffect } from 'react'
import { formatDuration } from '../time-utils'

/**
 * Returns a live elapsed time string (e.g. "23s") that ticks every second.
 * The value is computed directly from Date.now() - startTime each render,
 * with a 1s interval just to trigger re-renders. No state for the display
 * value itself, so no setState-during-render flicker.
 */
export function useElapsedTime(startTime: number | null): string | null {
  const [elapsedMs, setElapsedMs] = useState(0)

  useEffect(() => {
    if (startTime == null) {
      setElapsedMs(0)
      return
    }

    const updateElapsed = () => setElapsedMs(Date.now() - startTime)
    updateElapsed()

    const id = setInterval(updateElapsed, 1000)
    return () => clearInterval(id)
  }, [startTime])

  if (startTime == null) return null
  return formatDuration(elapsedMs)
}
