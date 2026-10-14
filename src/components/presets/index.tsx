import { useState } from 'react'
import { Check, Copy, GripVertical, Plus, Trash2 } from 'lucide-react'
import type { Preset } from '@/bindings'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import {
  useDeletePreset,
  useDuplicatePreset,
  usePresets,
  useProviders,
  useReorderPresets,
  useSavePreset,
  useSetActivePreset,
} from '@/hooks/usePresets'
import { cn } from '@/lib/utils'
import { PresetEditor } from './PresetEditor'

/** A preset the user is editing but has not saved yet. */
function blankPreset(): Preset {
  return {
    id: '',
    name: '',
    provider: 'open_ai',
    model: 'whisper-1',
    baseUrl: null,
    language: null,
    prompt: null,
    audioFormat: 'wav',
  }
}

/**
 * The presets screen.
 *
 * Several presets can point at the same provider with different models — that
 * is the whole point of the list, so activation is a per-row action rather than
 * a provider-level toggle.
 */
export function Presets() {
  const { data: config, isLoading } = usePresets()
  const { data: providers = [] } = useProviders()

  const savePreset = useSavePreset()
  const deletePreset = useDeletePreset()
  const duplicatePreset = useDuplicatePreset()
  const setActive = useSetActivePreset()
  const reorder = useReorderPresets()

  const [draft, setDraft] = useState<Preset | null>(null)
  const [dragId, setDragId] = useState<string | null>(null)

  const presets = config?.presets ?? []
  const activeId = config?.activeId ?? null

  const handleSaveDraft = async () => {
    if (!draft) return
    await savePreset.mutateAsync(draft)
    setDraft(null)
  }

  const handleDrop = (targetId: string) => {
    if (!dragId || dragId === targetId) return

    const ids = presets.map((p) => p.id)
    const from = ids.indexOf(dragId)
    const to = ids.indexOf(targetId)
    if (from < 0 || to < 0) return

    ids.splice(to, 0, ...ids.splice(from, 1))
    reorder.mutate(ids)
    setDragId(null)
  }

  if (isLoading) {
    return <p className="text-sm text-muted-foreground">Loading…</p>
  }

  return (
    <div className="space-y-6">
      <header className="flex items-start justify-between gap-4">
        <div>
          <h2 className="text-lg font-medium">Presets</h2>
          <p className="mt-1 text-sm text-muted-foreground">
            Each preset is one provider, model and set of options. Switch between them from the menu
            bar.
          </p>
        </div>
        <Button onClick={() => setDraft(blankPreset())} disabled={!!draft}>
          <Plus className="mr-2 h-4 w-4" />
          New preset
        </Button>
      </header>

      {presets.length === 0 && !draft && (
        <div className="rounded-xl border border-dashed border-border/60 p-10 text-center">
          <p className="text-sm text-muted-foreground">
            No presets yet. Create one to start dictating.
          </p>
        </div>
      )}

      <ul className="space-y-2">
        {presets.map((preset) => {
          const isEditing = draft?.id === preset.id
          const isActive = preset.id === activeId
          const provider = providers.find((p) => p.provider === preset.provider)

          return (
            <li
              key={preset.id}
              draggable={!draft}
              onDragStart={() => setDragId(preset.id)}
              onDragOver={(e) => e.preventDefault()}
              onDrop={() => handleDrop(preset.id)}
              className={cn(
                'rounded-xl border transition-colors',
                isActive ? 'border-primary/60 bg-primary/5' : 'border-border/60 bg-card/40',
                dragId === preset.id && 'opacity-50'
              )}
            >
              <div className="flex items-center gap-3 p-3">
                <GripVertical className="h-4 w-4 shrink-0 cursor-grab text-muted-foreground/40" />

                <button
                  type="button"
                  onClick={() => setActive.mutate(preset.id)}
                  className={cn(
                    'flex h-5 w-5 shrink-0 items-center justify-center rounded-full border transition-colors',
                    isActive
                      ? 'border-primary bg-primary text-primary-foreground'
                      : 'border-border hover:border-primary'
                  )}
                  aria-label={isActive ? 'Active preset' : `Activate ${preset.name}`}
                >
                  {isActive && <Check className="h-3 w-3" />}
                </button>

                <div className="min-w-0 flex-1">
                  <div className="flex items-center gap-2">
                    <span className="truncate font-medium">{preset.name}</span>
                    {isActive && (
                      <Badge variant="secondary" className="text-[10px]">
                        Active
                      </Badge>
                    )}
                  </div>
                  <p className="truncate text-xs text-muted-foreground">
                    {provider?.displayName ?? preset.provider}
                    {preset.model && ` · ${preset.model}`}
                    {preset.language && ` · ${preset.language}`}
                    {` · ${(preset.audioFormat ?? 'wav').toUpperCase()}`}
                  </p>
                </div>

                <Button
                  variant="ghost"
                  size="sm"
                  onClick={() => setDraft(isEditing ? null : preset)}
                >
                  {isEditing ? 'Close' : 'Edit'}
                </Button>
                <Button
                  variant="ghost"
                  size="icon"
                  onClick={() => duplicatePreset.mutate(preset.id)}
                  aria-label="Duplicate"
                >
                  <Copy className="h-4 w-4" />
                </Button>
                <Button
                  variant="ghost"
                  size="icon"
                  onClick={() => deletePreset.mutate(preset.id)}
                  aria-label="Delete"
                >
                  <Trash2 className="h-4 w-4" />
                </Button>
              </div>

              {isEditing && draft && (
                <div className="border-t border-border/60 p-4">
                  <PresetEditor
                    preset={draft}
                    providers={providers}
                    onChange={setDraft}
                    onSave={handleSaveDraft}
                    onCancel={() => setDraft(null)}
                    isSaving={savePreset.isPending}
                  />
                </div>
              )}
            </li>
          )
        })}
      </ul>

      {draft && draft.id === '' && (
        <div className="rounded-xl border border-border/60 bg-card/40 p-4">
          <PresetEditor
            preset={draft}
            providers={providers}
            onChange={setDraft}
            onSave={handleSaveDraft}
            onCancel={() => setDraft(null)}
            isSaving={savePreset.isPending}
          />
        </div>
      )}
    </div>
  )
}
