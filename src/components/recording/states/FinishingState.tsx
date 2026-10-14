import type { PopupStyle } from '@/bindings'
import { Orb } from '@/components/orb/Orb'

/**
 * Done — the text has been pasted.
 *
 * The orb flies apart and erodes. The backend keeps the window alive just long
 * enough for this to finish, so the popup dissolves rather than blinking out.
 */
export function FinishingState({ style }: { style: PopupStyle }) {
  return (
    <div className="flex h-full w-full items-center justify-center">
      <Orb style={style} state="dispersing" level={0} />
    </div>
  )
}
