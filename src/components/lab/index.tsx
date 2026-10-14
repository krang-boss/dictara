import { useEffect, useMemo, useState } from 'react'
import { Loader2, RotateCcw, Volume2 } from 'lucide-react'
import type { PopupStyle } from '@/bindings'
import { Orb, type OrbState } from '@/components/orb/Orb'
import { VARIANTS, styleFromVariant, type OrbVariant } from '@/components/orb/variants'
import { Button } from '@/components/ui/button'
import { Switch } from '@/components/ui/switch'
import { Label } from '@/components/ui/label'
import { usePopupStyle, useResetPopupStyle, useSavePopupStyle } from '@/hooks/usePopupStyle'
import { cn } from '@/lib/utils'
import { PaletteEditor } from './PaletteEditor'

/**
 * The popup laboratory.
 *
 * The recording orb is the one piece of UI seen on every dictation, so it gets
 * a screen of its own: pick a starting variant, then tune it live against a
 * simulated microphone level. Nothing is committed until "Use this" — the
 * preview is local state, so experimenting cannot break the real popup.
 */
export function Lab() {
  const { data: saved, isLoading } = usePopupStyle()
  const savePopupStyle = useSavePopupStyle()
  const resetPopupStyle = useResetPopupStyle()

  const [draft, setDraft] = useState<PopupStyle | null>(null)
  const [simulate, setSimulate] = useState(true)
  const [state, setState] = useState<OrbState>('recording')
  const [replayKey, setReplayKey] = useState(0)
  const level = useSimulatedLevel(simulate && state === 'recording')

  // The finish is a one-shot animation: it ends with nothing on screen, so the
  // preview replays it instead of leaving the stage blank.
  useEffect(() => {
    if (state !== 'dispersing') return
    const timer = setInterval(() => setReplayKey((key) => key + 1), 1400)
    return () => clearInterval(timer)
  }, [state])

  // Seed the draft once the stored style arrives.
  useEffect(() => {
    if (saved && !draft) setDraft(saved)
  }, [saved, draft])

  const dirty = useMemo(
    () => !!draft && !!saved && JSON.stringify(draft) !== JSON.stringify(saved),
    [draft, saved]
  )

  if (isLoading || !draft) {
    return <p className="text-sm text-muted-foreground">Loading…</p>
  }

  const update = (patch: Partial<PopupStyle>) => setDraft({ ...draft, ...patch })

  const pickVariant = (variant: OrbVariant) =>
    setDraft(styleFromVariant(variant, { size: draft.size, showTimer: draft.showTimer }))

  return (
    <div className="space-y-8">
      <header>
        <h2 className="text-lg font-medium">Recording popup</h2>
        <p className="mt-1 text-sm text-muted-foreground">
          Pick a starting point, then tune it. The preview reacts to a simulated voice level.
        </p>
      </header>

      {/* Stage */}
      <div className="relative overflow-hidden rounded-2xl border border-border/60">
        {/* A busy backdrop, because the popup floats over arbitrary content. */}
        <div className="absolute inset-0 bg-[radial-gradient(circle_at_20%_20%,#1e293b,transparent_55%),radial-gradient(circle_at_80%_70%,#312e81,transparent_55%),linear-gradient(120deg,#0b0f19,#111827)]" />
        <div className="relative flex flex-col items-center justify-center gap-4 py-14">
          <Orb
            // Remounting on replay restarts the one-shot finish animation.
            key={state === 'dispersing' ? replayKey : 'steady'}
            style={draft}
            state={state}
            level={level}
            size={draft.size}
          />
          {draft.showTimer && state === 'recording' && (
            <span className="font-mono text-sm tabular-nums text-white/70">0:07</span>
          )}
        </div>

        <div className="relative flex flex-wrap items-center justify-between gap-3 border-t border-white/10 px-4 py-2">
          <div className="flex items-center gap-1">
            {STATES.map((option) => (
              <button
                key={option.id}
                type="button"
                onClick={() => setState(option.id)}
                title={option.hint}
                className={cn(
                  'rounded-md px-2.5 py-1 text-xs transition-colors',
                  state === option.id
                    ? 'bg-white/15 text-white'
                    : 'text-white/50 hover:text-white/80'
                )}
              >
                {option.label}
              </button>
            ))}
          </div>

          <label className="flex items-center gap-2 text-xs text-white/70">
            <Volume2 className="h-3.5 w-3.5" />
            Simulate voice
            <Switch
              checked={simulate}
              onCheckedChange={setSimulate}
              disabled={state !== 'recording'}
            />
          </label>
        </div>
      </div>

      {/* Variants */}
      <VariantRow variants={VARIANTS} selected={draft.variant} onPick={pickVariant} />

      {/* Controls */}
      <section className="grid gap-6 md:grid-cols-2">
        <div className="space-y-4">
          <Slider
            label="Speed"
            value={draft.speed}
            onChange={(speed) => update({ speed })}
            hint="How fast the motion evolves"
          />
          <Slider
            label="Turbulence"
            value={draft.turbulence}
            onChange={(turbulence) => update({ turbulence })}
            hint="Wispy versus churning"
          />
          <Slider
            label="Softness"
            value={draft.softness}
            onChange={(softness) => update({ softness })}
            hint="Crisp disc versus diffuse cloud"
          />
        </div>
        <div className="space-y-4">
          <Slider
            label="Saturation"
            value={draft.saturation}
            onChange={(saturation) => update({ saturation })}
            hint="How far the colours sit from grey"
          />
          <Slider
            label="Reactivity"
            value={draft.reactivity}
            onChange={(reactivity) => update({ reactivity })}
            hint="How much your voice drives the motion"
          />
          <div className="space-y-1.5">
            <div className="flex items-baseline justify-between">
              <Label className="text-xs uppercase tracking-wide text-muted-foreground">Size</Label>
              <span className="font-mono text-xs text-muted-foreground">{draft.size}px</span>
            </div>
            <input
              type="range"
              min={48}
              max={320}
              step={4}
              value={draft.size}
              onChange={(e) => update({ size: Number(e.target.value) })}
              className="w-full accent-primary"
            />
            <p className="text-xs text-muted-foreground/70">
              The popup window is sized to match, plus a little padding.
            </p>
          </div>
        </div>
      </section>

      <PaletteEditor palette={draft.palette} onChange={(palette) => update({ palette })} />

      <div className="flex items-center gap-3 border-t border-border/60 pt-5">
        <label className="mr-auto flex items-center gap-2 text-sm">
          <Switch
            checked={draft.showTimer}
            onCheckedChange={(showTimer) => update({ showTimer })}
          />
          Show elapsed time
        </label>

        <Button
          variant="ghost"
          onClick={() => {
            resetPopupStyle.mutate(undefined, { onSuccess: setDraft })
          }}
        >
          <RotateCcw className="mr-2 h-4 w-4" />
          Reset
        </Button>
        <Button onClick={() => savePopupStyle.mutate(draft)} disabled={!dirty}>
          {savePopupStyle.isPending && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
          {dirty ? 'Use this' : 'Saved'}
        </Button>
      </div>
    </div>
  )
}

const STATES: { id: OrbState; label: string; hint: string }[] = [
  { id: 'idle', label: 'Idle', hint: 'Waiting, before you start speaking' },
  { id: 'recording', label: 'Recording', hint: 'Listening — reacts to your voice' },
  { id: 'processing', label: 'Processing', hint: 'Waiting on the transcription provider' },
  { id: 'dispersing', label: 'Finish', hint: 'Text pasted — the orb leaves' },
]

function VariantRow({
  variants,
  selected,
  onPick,
}: {
  variants: OrbVariant[]
  selected: string
  onPick: (variant: OrbVariant) => void
}) {
  return (
    <div className="grid grid-cols-3 gap-3 sm:grid-cols-5">
      {variants.map((variant) => (
        <button
          key={variant.id}
          type="button"
          onClick={() => onPick(variant)}
          title={variant.description}
          className={cn(
            'group flex flex-col items-center gap-2 rounded-xl border p-3 transition-colors',
            selected === variant.id
              ? 'border-primary bg-primary/10'
              : 'border-border/60 hover:border-border'
          )}
        >
          {/* Fixed size: the swatches are for comparing colour, not scale. */}
          <Orb
            style={styleFromVariant(variant, { size: 72, showTimer: false })}
            state="recording"
            level={0.35}
            size={72}
          />
          <span className="text-xs text-muted-foreground group-hover:text-foreground">
            {variant.name}
          </span>
        </button>
      ))}
    </div>
  )
}

function Slider({
  label,
  value,
  onChange,
  hint,
}: {
  label: string
  value: number
  onChange: (value: number) => void
  hint: string
}) {
  return (
    <div className="space-y-1.5">
      <div className="flex items-baseline justify-between">
        <Label className="text-xs uppercase tracking-wide text-muted-foreground">{label}</Label>
        <span className="font-mono text-xs text-muted-foreground">{value.toFixed(2)}</span>
      </div>
      <input
        type="range"
        min={0}
        max={1}
        step={0.01}
        value={value}
        onChange={(e) => onChange(Number(e.target.value))}
        className="w-full accent-primary"
      />
      <p className="text-xs text-muted-foreground/70">{hint}</p>
    </div>
  )
}

/**
 * A stand-in for microphone level.
 *
 * Layered sines with different periods read as speech far better than a plain
 * oscillation: bursts with pauses, rather than a metronome.
 */
function useSimulatedLevel(enabled: boolean) {
  const [level, setLevel] = useState(0)

  useEffect(() => {
    if (!enabled) {
      setLevel(0)
      return
    }

    let frame = 0
    const start = performance.now()

    const tick = (now: number) => {
      const t = (now - start) / 1000
      const envelope = Math.max(0, Math.sin(t * 0.55) * 0.5 + 0.5)
      const syllables = Math.abs(Math.sin(t * 4.2)) * 0.6 + Math.abs(Math.sin(t * 7.9)) * 0.25
      setLevel(Math.min(1, envelope * syllables * 1.4))
      frame = requestAnimationFrame(tick)
    }

    frame = requestAnimationFrame(tick)
    return () => cancelAnimationFrame(frame)
  }, [enabled])

  return level
}
