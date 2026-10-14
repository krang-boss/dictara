import type { PopupStyle } from '@/bindings'
import { Orb } from '@/components/orb/Orb'

/**
 * Waiting on the provider.
 *
 * The same orb, spun up into a tight fast rotation. Recognisably the same
 * object, clearly no longer listening — the motion carries the difference, so
 * the colours stay the user's own rather than being overridden with grey.
 */
export function TranscribingState({ style }: { style: PopupStyle }) {
  return (
    <div className="flex h-full w-full items-center justify-center">
      <Orb style={style} state="processing" level={0} />
    </div>
  )
}
