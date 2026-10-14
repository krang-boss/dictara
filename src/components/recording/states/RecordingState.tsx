import { Square } from 'lucide-react'
import type { PopupStyle } from '@/bindings'
import { Orb } from '@/components/orb/Orb'

interface RecordingStateProps {
  elapsedMs: number
  smoothedLevel: number
  style: PopupStyle
  onStop: () => void
  isStopPending: boolean
}

/**
 * Recording, in progress.
 *
 * The orb carries the whole state: it is alive, it reacts to your voice, and
 * it is the only thing visible at rest. Controls fade in on hover so the
 * default appearance stays a single moving shape over your work.
 */
export function RecordingState({
  elapsedMs,
  smoothedLevel,
  style,
  onStop,
  isStopPending,
}: RecordingStateProps) {
  return (
    <div className="group relative flex h-full w-full flex-col items-center justify-center gap-1">
      <Orb style={style} state="recording" level={smoothedLevel} />

      {style.showTimer && (
        <span className="font-mono text-[11px] tabular-nums text-white/70 transition-opacity group-hover:opacity-0">
          {formatTime(elapsedMs)}
        </span>
      )}

      {/*
        Stop only. Cancelling is Escape — a global shortcut, so it works without
        the popup ever taking focus, which it deliberately never does.
      */}
      <div className="absolute inset-0 flex items-center justify-center opacity-0 transition-opacity duration-150 group-hover:opacity-100">
        <button
          onClick={onStop}
          disabled={isStopPending}
          aria-label="Stop and transcribe"
          className="flex h-8 w-8 items-center justify-center rounded-full bg-black/60 transition-colors hover:bg-black/80 disabled:opacity-50"
        >
          <Square className="h-2.5 w-2.5 text-white" fill="white" strokeWidth={0} />
        </button>
      </div>
    </div>
  )
}

function formatTime(ms: number): string {
  const totalSeconds = Math.floor(ms / 1000)
  const minutes = Math.floor(totalSeconds / 60)
  const seconds = totalSeconds % 60
  return `${minutes}:${seconds.toString().padStart(2, '0')}`
}
