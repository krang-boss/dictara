import { useState } from 'react'
import { Clipboard, Search, Trash2 } from 'lucide-react'
import type { HistoryEntry } from '@/bindings'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import {
  useClearHistory,
  useDeleteHistoryEntry,
  useHistory,
  useHistoryStats,
  usePasteText,
} from '@/hooks/useHistory'

/**
 * The history screen.
 *
 * Only text is kept — audio is never written to disk — so every row is
 * searchable and cheap. Pasting a row sends it to whichever app is frontmost,
 * which is why the window has to give up focus first (handled in the backend).
 */
export function History() {
  const [search, setSearch] = useState('')
  const { data: entries = [], isLoading } = useHistory({ search: search || null })
  const { data: stats } = useHistoryStats()

  const deleteEntry = useDeleteHistoryEntry()
  const clearHistory = useClearHistory()
  const pasteText = usePasteText()

  const [confirmingClear, setConfirmingClear] = useState(false)

  return (
    <div className="space-y-5">
      <header className="flex items-start justify-between gap-4">
        <div>
          <h2 className="text-lg font-medium">History</h2>
          <p className="mt-1 text-sm text-muted-foreground">
            {stats
              ? `${stats.totalEntries} recordings · ${stats.totalWords.toLocaleString()} words · ${formatDuration(stats.totalDurationMs)} dictated`
              : 'Text only — audio is never stored.'}
          </p>
        </div>

        {entries.length > 0 &&
          (confirmingClear ? (
            <div className="flex items-center gap-2">
              <Button
                variant="destructive"
                size="sm"
                onClick={() => {
                  clearHistory.mutate(undefined)
                  setConfirmingClear(false)
                }}
              >
                Delete everything
              </Button>
              <Button variant="ghost" size="sm" onClick={() => setConfirmingClear(false)}>
                Cancel
              </Button>
            </div>
          ) : (
            <Button variant="ghost" size="sm" onClick={() => setConfirmingClear(true)}>
              Clear
            </Button>
          ))}
      </header>

      <div className="relative">
        <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
        <Input
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          placeholder="Search transcriptions…"
          className="pl-9"
        />
      </div>

      {isLoading && <p className="text-sm text-muted-foreground">Loading…</p>}

      {!isLoading && entries.length === 0 && (
        <div className="rounded-xl border border-dashed border-border/60 p-10 text-center">
          <p className="text-sm text-muted-foreground">
            {search ? 'Nothing matches that search.' : 'Nothing dictated yet.'}
          </p>
        </div>
      )}

      <ul className="space-y-2">
        {entries.map((entry) => (
          <HistoryRow
            key={entry.id}
            entry={entry}
            onPaste={() => pasteText.mutate(entry.text)}
            onDelete={() => deleteEntry.mutate(entry.id)}
          />
        ))}
      </ul>
    </div>
  )
}

function HistoryRow({
  entry,
  onPaste,
  onDelete,
}: {
  entry: HistoryEntry
  onPaste: () => void
  onDelete: () => void
}) {
  const [copied, setCopied] = useState(false)

  const handleCopy = async () => {
    await navigator.clipboard.writeText(entry.text)
    setCopied(true)
    setTimeout(() => setCopied(false), 1200)
  }

  return (
    <li className="group rounded-xl border border-border/60 bg-card/40 p-3">
      <p className="whitespace-pre-wrap text-sm leading-relaxed">{entry.text}</p>

      <div className="mt-2 flex items-center gap-2 text-xs text-muted-foreground">
        <span>{formatTimestamp(entry.createdAt)}</span>
        {entry.presetName && <span>· {entry.presetName}</span>}
        {entry.appName && <span>· {entry.appName}</span>}
        <span>· {formatDuration(entry.durationMs)}</span>

        <div className="ml-auto flex items-center gap-1 opacity-0 transition-opacity group-hover:opacity-100">
          <Button variant="ghost" size="sm" onClick={handleCopy}>
            <Clipboard className="mr-1 h-3 w-3" />
            {copied ? 'Copied' : 'Copy'}
          </Button>
          <Button variant="ghost" size="sm" onClick={onPaste}>
            Paste
          </Button>
          <Button variant="ghost" size="icon" onClick={onDelete} aria-label="Delete">
            <Trash2 className="h-3.5 w-3.5" />
          </Button>
        </div>
      </div>
    </li>
  )
}

export function formatTimestamp(seconds: number) {
  const date = new Date(seconds * 1000)
  const today = new Date()
  const sameDay = date.toDateString() === today.toDateString()

  return sameDay
    ? date.toLocaleTimeString(undefined, { hour: 'numeric', minute: '2-digit' })
    : date.toLocaleDateString(undefined, {
        month: 'short',
        day: 'numeric',
        hour: 'numeric',
        minute: '2-digit',
      })
}

export function formatDuration(ms: number) {
  const totalSeconds = Math.round(ms / 1000)
  if (totalSeconds < 60) return `${totalSeconds}s`

  const minutes = Math.floor(totalSeconds / 60)
  if (minutes < 60) return `${minutes}m ${totalSeconds % 60}s`

  return `${Math.floor(minutes / 60)}h ${minutes % 60}m`
}
