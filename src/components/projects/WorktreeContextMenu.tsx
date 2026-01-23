import { useState } from 'react'
import {
  Archive,
  Code,
  FileJson,
  FolderOpen,
  Play,
  Terminal,
  Trash2,
  X,
} from 'lucide-react'
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from '@/components/ui/alert-dialog'
import {
  ContextMenu,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuSeparator,
  ContextMenuTrigger,
} from '@/components/ui/context-menu'
import { isBaseSession, type Worktree } from '@/types/projects'
import {
  useArchiveWorktree,
  useCloseBaseSession,
  useDeleteWorktree,
  useOpenWorktreeInEditor,
  useOpenWorktreeInFinder,
  useOpenWorktreeInTerminal,
  useRunScript,
} from '@/services/projects'
import { usePreferences } from '@/services/preferences'
import { getEditorLabel, getTerminalLabel } from '@/types/preferences'
import { useTerminalStore } from '@/store/terminal-store'
import { getFileManagerName } from '@/lib/platform'

interface WorktreeContextMenuProps {
  worktree: Worktree
  projectId: string
  projectPath: string
  children: React.ReactNode
}

export function WorktreeContextMenu({
  worktree,
  projectId,
  projectPath,
  children,
}: WorktreeContextMenuProps) {
  const [showDeleteConfirm, setShowDeleteConfirm] = useState(false)
  const archiveWorktree = useArchiveWorktree()
  const closeBaseSession = useCloseBaseSession()
  const deleteWorktree = useDeleteWorktree()
  const openInFinder = useOpenWorktreeInFinder()
  const openInTerminal = useOpenWorktreeInTerminal()
  const openInEditor = useOpenWorktreeInEditor()
  const { data: runScript } = useRunScript(worktree.path)
  const { data: preferences } = usePreferences()
  const isBase = isBaseSession(worktree)

  // Suppress unused variable warning
  void projectPath

  const handleRun = () => {
    if (runScript) {
      useTerminalStore.getState().startRun(worktree.id, runScript)
    }
  }

  const handleOpenTerminalPanel = () => {
    useTerminalStore.getState().addTerminal(worktree.id)
  }

  const handleOpenInFinder = () => {
    openInFinder.mutate(worktree.path)
  }

  const handleOpenInTerminal = () => {
    openInTerminal.mutate({
      worktreePath: worktree.path,
      terminal: preferences?.terminal,
    })
  }

  const handleOpenInEditor = () => {
    openInEditor.mutate({
      worktreePath: worktree.path,
      editor: preferences?.editor,
    })
  }

  const handleArchiveOrClose = () => {
    if (isBase) {
      // Base sessions are closed (removed from list), not archived
      closeBaseSession.mutate({ worktreeId: worktree.id, projectId })
    } else {
      // Regular worktrees are archived (can be restored later)
      archiveWorktree.mutate({ worktreeId: worktree.id, projectId })
    }
  }

  const handleDelete = () => {
    deleteWorktree.mutate({ worktreeId: worktree.id, projectId })
    setShowDeleteConfirm(false)
  }

  const handleOpenJeanConfig = () => {
    openInEditor.mutate({
      worktreePath: `${worktree.path}/jean.json`,
      editor: preferences?.editor,
    })
  }

  return (
    <ContextMenu>
      <ContextMenuTrigger asChild>{children}</ContextMenuTrigger>
      <ContextMenuContent className="w-48">
        <ContextMenuItem onClick={handleOpenTerminalPanel}>
          <Terminal className="mr-2 h-4 w-4" />
          Terminal
        </ContextMenuItem>

        {runScript && (
          <ContextMenuItem onClick={handleRun}>
            <Play className="mr-2 h-4 w-4" />
            Run
          </ContextMenuItem>
        )}

        <ContextMenuItem onClick={handleOpenJeanConfig}>
          <FileJson className="mr-2 h-4 w-4" />
          Edit jean.json
        </ContextMenuItem>

        <ContextMenuSeparator />

        <ContextMenuItem onClick={handleOpenInEditor}>
          <Code className="mr-2 h-4 w-4" />
          Open in {getEditorLabel(preferences?.editor)}
        </ContextMenuItem>

        <ContextMenuItem onClick={handleOpenInFinder}>
          <FolderOpen className="mr-2 h-4 w-4" />
          Open in {getFileManagerName()}
        </ContextMenuItem>

        <ContextMenuItem onClick={handleOpenInTerminal}>
          <Terminal className="mr-2 h-4 w-4" />
          Open in {getTerminalLabel(preferences?.terminal)}
        </ContextMenuItem>

        <ContextMenuSeparator />

        <ContextMenuItem onClick={handleArchiveOrClose}>
          {isBase ? (
            <>
              <X className="mr-2 h-4 w-4" />
              Close Session
            </>
          ) : (
            <>
              <Archive className="mr-2 h-4 w-4" />
              Archive Worktree
            </>
          )}
        </ContextMenuItem>

        {!isBase && (
          <ContextMenuItem onClick={() => setShowDeleteConfirm(true)}>
            <Trash2 className="mr-2 h-4 w-4 text-destructive" />
            Delete Worktree
          </ContextMenuItem>
        )}
      </ContextMenuContent>

      <AlertDialog open={showDeleteConfirm} onOpenChange={setShowDeleteConfirm}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Delete Worktree</AlertDialogTitle>
            <AlertDialogDescription>
              This will permanently delete the worktree, its branch, and all
              associated sessions. This action cannot be undone.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>Cancel</AlertDialogCancel>
            <AlertDialogAction
              onClick={handleDelete}
              className="bg-destructive text-destructive-foreground hover:bg-destructive/90"
            >
              Delete
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </ContextMenu>
  )
}
