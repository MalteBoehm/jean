import { type HTMLProps, useEffect, useState } from 'react'
import { cn } from '@/lib/utils'
import { Icons } from './WindowControlIcons'
import { useCommandContext } from '@/hooks/use-command-context'
import { executeCommand } from '@/lib/commands'
import { getCurrentWindow } from '@tauri-apps/api/window'

interface WindowsWindowControlsProps extends HTMLProps<HTMLDivElement> {
  className?: string
}

export function WindowsWindowControls({
  className,
  ...props
}: WindowsWindowControlsProps) {
  const context = useCommandContext()
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

  const handleClose = async () => {
    await executeCommand('window-close', context)
  }

  const handleMinimize = async () => {
    await executeCommand('window-minimize', context)
  }

  const handleMaximizeRestore = async () => {
    await executeCommand('window-toggle-maximize', context)
    setIsMaximized(!isMaximized)
  }

  const buttonBaseClass =
    'flex h-8 w-11 items-center justify-center text-foreground/80 hover:bg-foreground/10 transition-colors'

  return (
    <div className={cn('flex items-center', className)} {...props}>
      <button
        type="button"
        onClick={handleMinimize}
        className={buttonBaseClass}
        aria-label="Minimize"
      >
        <Icons.minWin className="h-[10px] w-[10px]" />
      </button>
      <button
        type="button"
        onClick={handleMaximizeRestore}
        className={buttonBaseClass}
        aria-label={isMaximized ? 'Restore' : 'Maximize'}
      >
        {isMaximized ? (
          <Icons.restoreWin className="h-[10px] w-[10px]" />
        ) : (
          <Icons.maxWin className="h-[10px] w-[10px]" />
        )}
      </button>
      <button
        type="button"
        onClick={handleClose}
        className={cn(buttonBaseClass, 'hover:bg-red-500 hover:text-white')}
        aria-label="Close"
      >
        <Icons.closeWin className="h-[10px] w-[10px]" />
      </button>
    </div>
  )
}

export default WindowsWindowControls
