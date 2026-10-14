import { useState } from 'react'
import { Check, Clipboard, History, Power, Settings, Sparkles } from 'lucide-react'
import { commands } from '@/bindings'
import { Orb } from '@/components/orb/Orb'
import { formatTimestamp } from '@/components/history'
import { useHistory, usePasteText } from '@/hooks/useHistory'
import { usePopupStyle } from '@/hooks/usePopupStyle'
import { usePresets, useSetActivePreset } from '@/hooks/usePresets'
import { cn } from '@/lib/utils'

const RECENT_COUNT = 6

/**
 * The menu bar panel.
 *
 * Replaces what used to be a three-item native menu: switch preset, reach the
 * last few transcriptions, and get to preferences.
 *
 * Plain and opaque on purpose. Everything here uses the ordinary theme tokens
 * — no translucency, no text shadows, no per-window contrast tuning — because
 * the panel is something you glance at several times a day and legibility
 * beats effect.
 */
export function MenuBarPanel() {
  const { data: presetsConfig } = usePresets()
  const setActive = useSetActivePreset()
  const { data: style } = usePopupStyle()
  const { data: recent = [] } = useHistory({ limit: RECENT_COUNT })
  const pasteText = usePasteText()

  const presets = presetsConfig?.presets ?? []
  const activeId = presetsConfig?.activeId ?? null
  const active = presets.find((p) => p.id === activeId)

  return (
    <div className="flex h-screen flex-col overflow-hidden text-[14px] text-foreground">
      {/* The whole strip is a drag handle, the way a real popover title bar is. */}
      <header className="flex items-center gap-3 px-4 pb-3 pt-4" data-tauri-drag-region>
        {style && <Orb style={style} state="idle" level={0} size={36} />}
        <div className="min-w-0">
          <p className="truncate text-[15px] font-semibold leading-tight">VoiceFlow</p>
          <p className="truncate text-[12px] text-muted-foreground">
            {active ? `${active.name} · ${active.model || active.provider}` : 'No preset selected'}
          </p>
        </div>
      </header>

      <Section title="Presets" icon={Sparkles}>
        {presets.length === 0 && (
          <p className="px-3 py-2 text-[13px] text-muted-foreground">
            No presets yet — add one in Preferences.
          </p>
        )}
        {presets.map((preset) => (
          <button
            key={preset.id}
            type="button"
            onClick={() => setActive.mutate(preset.id)}
            className={cn(
              'flex w-full items-center gap-2 rounded-lg px-3 py-2 text-left transition-colors',
              preset.id === activeId ? 'bg-accent font-medium' : 'hover:bg-accent/60'
            )}
          >
            <Check
              className={cn(
                'h-4 w-4 shrink-0',
                preset.id === activeId ? 'text-primary' : 'text-transparent'
              )}
            />
            <span className="min-w-0 flex-1 truncate">{preset.name}</span>
            <span className="shrink-0 text-[12px] uppercase tracking-wide text-muted-foreground">
              {preset.audioFormat}
            </span>
          </button>
        ))}
      </Section>

      <Section title="Recent" icon={History} className="min-h-0 flex-1">
        {recent.length === 0 && (
          <p className="px-3 py-2 text-[13px] text-muted-foreground">Nothing dictated yet.</p>
        )}
        <div className="min-h-0 flex-1 overflow-y-auto">
          {recent.map((entry) => (
            <RecentRow key={entry.id} entry={entry} onPaste={() => pasteText.mutate(entry.text)} />
          ))}
        </div>
      </Section>

      <footer className="mt-auto flex items-center gap-1 border-t border-border px-2 py-1.5">
        <FooterButton
          icon={Settings}
          label="Preferences"
          onClick={() => {
            commands.openPreferences()
            commands.hideMenubarPanel()
          }}
        />
        <FooterButton
          icon={Power}
          label="Quit"
          onClick={() => commands.quitApp()}
          className="ml-auto"
        />
      </footer>
    </div>
  )
}

function RecentRow({
  entry,
  onPaste,
}: {
  entry: { id: number; text: string; createdAt: number }
  onPaste: () => void
}) {
  const [copied, setCopied] = useState(false)

  const handleCopy = async (e: React.MouseEvent) => {
    e.stopPropagation()
    await navigator.clipboard.writeText(entry.text)
    setCopied(true)
    setTimeout(() => setCopied(false), 1200)
  }

  return (
    <button
      type="button"
      onClick={onPaste}
      title="Paste into the frontmost app"
      className="group flex w-full items-start gap-2 rounded-lg px-3 py-2 text-left transition-colors hover:bg-accent/60"
    >
      <span className="line-clamp-2 min-w-0 flex-1 text-[13px] leading-snug">{entry.text}</span>
      <span className="shrink-0 pt-0.5 text-[11px] text-muted-foreground group-hover:hidden">
        {formatTimestamp(entry.createdAt)}
      </span>
      <span
        onClick={handleCopy}
        className="hidden shrink-0 pt-0.5 text-muted-foreground hover:text-foreground group-hover:block"
      >
        {copied ? <Check className="h-3.5 w-3.5" /> : <Clipboard className="h-3.5 w-3.5" />}
      </span>
    </button>
  )
}

function Section({
  title,
  icon: Icon,
  children,
  className,
}: {
  title: string
  icon: React.ComponentType<{ className?: string }>
  children: React.ReactNode
  className?: string
}) {
  return (
    <section className={cn('flex flex-col px-2 pb-2', className)}>
      <h2 className="flex items-center gap-1.5 px-3 py-1.5 text-[11px] font-semibold uppercase tracking-wider text-muted-foreground">
        <Icon className="h-3.5 w-3.5" />
        {title}
      </h2>
      {children}
    </section>
  )
}

function FooterButton({
  icon: Icon,
  label,
  onClick,
  className,
}: {
  icon: React.ComponentType<{ className?: string }>
  label: string
  onClick: () => void
  className?: string
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      className={cn(
        'flex items-center gap-1.5 rounded-lg px-2.5 py-2 text-[13px] text-muted-foreground transition-colors hover:bg-accent/60 hover:text-foreground',
        className
      )}
    >
      <Icon className="h-3.5 w-3.5" />
      {label}
    </button>
  )
}
