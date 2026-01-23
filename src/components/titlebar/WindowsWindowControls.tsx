import type { HTMLProps } from 'react'
import { cn } from '@/lib/utils'
import { Icons } from './WindowControlIcons'
import { useCommandContext } from '@/hooks/use-command-context'
import { executeCommand } from '@/lib/commands'
import { useWindowMaximized } from '@/hooks/use-window-maximized'

interface WindowsWindowControlsProps extends HTMLProps<HTMLDivElement> {
  className?: string
}

export function WindowsWindowControls({
  className,
  ...props
}: WindowsWindowControlsProps) {
  const context = useCommandContext()
  const isMaximized = useWindowMaximized()

  const handleClose = async () => {
    await executeCommand('window-close', context)
  }

  const handleMinimize = async () => {
    await executeCommand('window-minimize', context)
  }

  const handleMaximizeRestore = async () => {
    await executeCommand('window-toggle-maximize', context)
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
