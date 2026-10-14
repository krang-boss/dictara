import { useCallback, useRef } from 'react'
import './RecordingPopup.css'
import {
  useRecordingStateMachine,
  type RecordingStateChanged,
} from './hooks/useRecordingStateMachine'
import { useAudioLevel } from './hooks/useAudioLevel'
import { useRecordingTimer } from './hooks/useRecordingTimer'
import { useResizePopupForError } from '@/hooks/useRecording'
import { usePopupStyle } from '@/hooks/usePopupStyle'
import { RecordingState } from './states/RecordingState'
import { TranscribingState } from './states/TranscribingState'
import { FinishingState } from './states/FinishingState'
import { ErrorState } from './states/ErrorState'

function RecordingPopup() {
  const { smoothedLevel } = useAudioLevel()
  const resizePopupForError = useResizePopupForError()
  const { data: style } = usePopupStyle()

  // Refs for timer functions (to break circular dependency)
  const timerFunctionsRef = useRef<{
    startTimer: () => void
    cleanupTimer: () => void
  } | null>(null)

  // Event handler for recording state changes (side effects)
  const handleRecordingEvent = useCallback(
    (event: RecordingStateChanged) => {
      const timerFns = timerFunctionsRef.current
      if (!timerFns) return

      switch (event.state) {
        case 'started':
          timerFns.startTimer()
          break

        case 'transcribing':
        case 'stopped':
        case 'cancelled':
          timerFns.cleanupTimer()
          break

        case 'error':
          timerFns.cleanupTimer()
          resizePopupForError.mutate()
          break
      }
    },
    [resizePopupForError]
  )

  const {
    state,
    error,
    handleStop,
    handleRetry,
    handleDismiss,
    isStopPending,
    isRetryPending,
    isDismissPending,
  } = useRecordingStateMachine(handleRecordingEvent)

  const { elapsedMs, startTimer, cleanupTimer } = useRecordingTimer(handleStop)

  // Update refs with timer functions
  timerFunctionsRef.current = { startTimer, cleanupTimer }

  // The error state needs a readable surface; the orb states are meant to float
  // free over whatever is behind them, with no chrome at all.
  const isError = state === 'error' && !!error

  return (
    <div
      className={
        isError
          ? 'h-screen w-screen overflow-hidden rounded-2xl border border-white/10 bg-neutral-900/95 font-sans backdrop-blur-xl'
          : 'h-screen w-screen overflow-hidden font-sans'
      }
    >
      {isError && (
        <ErrorState
          error={error}
          onRetry={handleRetry}
          onDismiss={handleDismiss}
          isRetryPending={isRetryPending}
          isDismissPending={isDismissPending}
        />
      )}

      {state === 'transcribing' && style && <TranscribingState style={style} />}

      {state === 'finishing' && style && <FinishingState style={style} />}

      {state === 'recording' && style && (
        <RecordingState
          elapsedMs={elapsedMs}
          smoothedLevel={smoothedLevel}
          style={style}
          onStop={handleStop}
          isStopPending={isStopPending}
        />
      )}
    </div>
  )
}

export default RecordingPopup
