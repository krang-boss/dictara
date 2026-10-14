import { useEffect, useState } from 'react'
import { Check, Loader2, TriangleAlert } from 'lucide-react'
import type { Preset, Provider, ProviderInfo } from '@/bindings'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Textarea } from '@/components/ui/textarea'
import { useCredentials, useSaveCredentials } from '@/hooks/useCredentials'
import { useTestPreset } from '@/hooks/usePresets'
import { cn } from '@/lib/utils'

const MASKED_KEY = '••••••••••••••••'

const AUDIO_FORMATS = [
  { id: 'wav' as const, hint: 'Lossless, large' },
  { id: 'mp3' as const, hint: '64 kbps mono, ~8× smaller' },
]

interface PresetEditorProps {
  preset: Preset
  providers: ProviderInfo[]
  onChange: (preset: Preset) => void
  onSave: () => void
  onCancel: () => void
  isSaving: boolean
}

/**
 * Form for one preset.
 *
 * The API key is edited here but stored per *provider*, not per preset — two
 * presets pointing at OpenAI share one key. The copy says so, because it is the
 * one part of the model that is not obvious from the UI.
 */
export function PresetEditor({
  preset,
  providers,
  onChange,
  onSave,
  onCancel,
  isSaving,
}: PresetEditorProps) {
  const info = providers.find((p) => p.provider === preset.provider)
  const { data: credentials } = useCredentials()
  const saveCredentials = useSaveCredentials()
  const testPreset = useTestPreset()

  const hasKey = credentials?.find((c) => c.provider === preset.provider)?.configured ?? false
  const [apiKey, setApiKey] = useState('')
  const [testResult, setTestResult] = useState<'ok' | 'rejected' | null>(null)

  // A new provider means a different stored key; clear whatever was typed.
  useEffect(() => {
    setApiKey('')
    setTestResult(null)
  }, [preset.provider])

  const handleProviderChange = (provider: Provider) => {
    const next = providers.find((p) => p.provider === provider)
    onChange({
      ...preset,
      provider,
      model: next?.defaultModel ?? '',
      baseUrl: next?.requiresBaseUrl ? (preset.baseUrl ?? '') : null,
    })
  }

  const handleTest = async () => {
    setTestResult(null)
    if (apiKey.trim()) {
      await saveCredentials.mutateAsync({ provider: preset.provider, apiKey })
      setApiKey('')
    }
    const ok = await testPreset.mutateAsync(preset)
    setTestResult(ok ? 'ok' : 'rejected')
  }

  const handleSave = async () => {
    if (apiKey.trim()) {
      await saveCredentials.mutateAsync({ provider: preset.provider, apiKey })
      setApiKey('')
    }
    onSave()
  }

  const missingKey = (info?.requiresApiKey ?? true) && !hasKey && !apiKey.trim()
  const missingUrl = (info?.requiresBaseUrl ?? false) && !preset.baseUrl?.trim()
  const missingModel = (info?.requiresModel ?? true) && !preset.model?.trim()
  const canSave = !!preset.name.trim() && !missingUrl && !missingModel

  return (
    <div className="space-y-5">
      <Field label="Name" hint="How this preset appears in the menu bar.">
        <Input
          value={preset.name}
          onChange={(e) => onChange({ ...preset, name: e.target.value })}
          placeholder="Fast English"
          autoFocus
        />
      </Field>

      <Field label="Provider">
        <div className="grid grid-cols-2 gap-2 sm:grid-cols-3">
          {providers.map((option) => (
            <button
              key={option.provider}
              type="button"
              onClick={() => handleProviderChange(option.provider)}
              className={cn(
                'rounded-lg border px-3 py-2 text-left text-sm transition-colors',
                preset.provider === option.provider
                  ? 'border-primary bg-primary/10 text-foreground'
                  : 'border-border/60 text-muted-foreground hover:border-border hover:text-foreground'
              )}
            >
              {option.displayName}
            </button>
          ))}
        </div>
      </Field>

      {info?.requiresBaseUrl && (
        <Field
          label={preset.provider === 'azure_open_ai' ? 'Endpoint' : 'Base URL'}
          hint={
            preset.provider === 'azure_open_ai'
              ? 'Full deployment path, without the api-version parameter.'
              : 'Root of an OpenAI-compatible API. /audio/transcriptions is appended.'
          }
        >
          <Input
            value={preset.baseUrl ?? ''}
            onChange={(e) => onChange({ ...preset, baseUrl: e.target.value })}
            placeholder={
              preset.provider === 'azure_open_ai'
                ? 'https://xxx.openai.azure.com/openai/deployments/whisper/audio/transcriptions'
                : 'http://localhost:8080/v1'
            }
          />
        </Field>
      )}

      {info?.requiresModel && (
        <Field label="Model">
          <Input
            value={preset.model ?? ''}
            onChange={(e) => onChange({ ...preset, model: e.target.value })}
            placeholder={info.defaultModel}
          />
        </Field>
      )}

      <Field
        label={info?.requiresApiKey ? 'API key' : 'API key (optional)'}
        hint={`Stored in the macOS Keychain, shared by every ${info?.displayName ?? ''} preset.`}
      >
        <Input
          type="password"
          value={apiKey}
          onChange={(e) => setApiKey(e.target.value)}
          placeholder={hasKey ? MASKED_KEY : 'sk-…'}
        />
      </Field>

      <Field
        label="Upload format"
        hint="MP3 is roughly 8× smaller than WAV, so uploads finish sooner and you send far less data per recording. Set per preset."
      >
        <div className="grid grid-cols-2 gap-2">
          {AUDIO_FORMATS.map((format) => (
            <button
              key={format.id}
              type="button"
              onClick={() => onChange({ ...preset, audioFormat: format.id })}
              className={cn(
                'rounded-lg border px-3 py-2 text-left transition-colors',
                preset.audioFormat === format.id
                  ? 'border-primary bg-primary/10 text-foreground'
                  : 'border-border/60 text-muted-foreground hover:border-border hover:text-foreground'
              )}
            >
              <span className="block text-sm font-medium uppercase">{format.id}</span>
              <span className="block text-xs text-muted-foreground">{format.hint}</span>
            </button>
          ))}
        </div>
      </Field>

      <Field label="Language" hint="ISO code, or blank to auto-detect.">
        <Input
          value={preset.language ?? ''}
          onChange={(e) => onChange({ ...preset, language: e.target.value || null })}
          placeholder="auto"
        />
      </Field>

      <Field label="Prompt" hint="Names and jargon to bias the transcription toward.">
        <Textarea
          value={preset.prompt ?? ''}
          onChange={(e) => onChange({ ...preset, prompt: e.target.value || null })}
          placeholder="Kubernetes, Anthropic, Igor…"
          rows={2}
        />
      </Field>

      {missingKey && (
        <p className="flex items-center gap-2 text-sm text-amber-500">
          <TriangleAlert className="h-4 w-4 shrink-0" />
          No API key stored for {info?.displayName}. This preset will fail until you add one.
        </p>
      )}

      {testResult === 'ok' && (
        <p className="flex items-center gap-2 text-sm text-emerald-500">
          <Check className="h-4 w-4" /> Works.
        </p>
      )}
      {testResult === 'rejected' && (
        <p className="flex items-center gap-2 text-sm text-destructive">
          <TriangleAlert className="h-4 w-4" /> Credentials rejected.
        </p>
      )}
      {testPreset.isError && (
        <p className="text-sm text-destructive">{(testPreset.error as Error).message}</p>
      )}

      <div className="flex items-center gap-2 pt-1">
        <Button onClick={handleSave} disabled={!canSave || isSaving}>
          {isSaving && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
          Save
        </Button>
        <Button variant="ghost" onClick={onCancel}>
          Cancel
        </Button>
        <Button
          variant="outline"
          className="ml-auto"
          onClick={handleTest}
          disabled={testPreset.isPending || !canSave}
        >
          {testPreset.isPending && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
          Test
        </Button>
      </div>
    </div>
  )
}

function Field({
  label,
  hint,
  children,
}: {
  label: string
  hint?: string
  children: React.ReactNode
}) {
  return (
    <div className="space-y-1.5">
      <Label className="text-xs font-medium uppercase tracking-wide text-muted-foreground">
        {label}
      </Label>
      {children}
      {hint && <p className="text-xs text-muted-foreground/80">{hint}</p>}
    </div>
  )
}
