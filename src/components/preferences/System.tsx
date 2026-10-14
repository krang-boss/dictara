import { error as logError } from '@tauri-apps/plugin-log'
import { RotateCcw } from 'lucide-react'
import { Switch } from '../ui/switch'
import { Label } from '../ui/label'
import { Button } from '../ui/button'
import { useIsAutostartEnabled, useToggleAutostart } from '@/hooks/useAutostart'
import { useRestartOnboarding } from '@/hooks/useOnboardingNavigation'
import { useAppConfig } from '@/hooks/useAppConfig'
import { useSaveAppConfig } from '@/hooks/useSaveAppConfig'

const RETENTION_OPTIONS = [
  { days: 30, label: '30 days' },
  { days: 90, label: '90 days' },
  { days: 365, label: '1 year' },
  // 0 disables pruning entirely.
  { days: 0, label: 'Forever' },
]

export function System() {
  const { data: isEnabled, isLoading: isCheckingStatus } = useIsAutostartEnabled()
  const { toggle, isLoading: isToggling } = useToggleAutostart()
  const restartOnboarding = useRestartOnboarding()
  const { data: appConfig } = useAppConfig()
  const saveAppConfig = useSaveAppConfig()

  const handleToggleAutostart = async (checked: boolean) => {
    try {
      await toggle(!checked) // Toggle to the opposite of current state
    } catch (e) {
      logError(`[System] Failed to toggle autostart: ${e}`)
    }
  }

  const handleRetentionChange = async (days: number) => {
    try {
      await saveAppConfig.mutateAsync({ historyRetentionDays: days })
    } catch (e) {
      logError(`[System] Failed to update history retention: ${e}`)
    }
  }

  return (
    <div className="space-y-4">
      <div className="space-y-2">
        <p className="text-sm text-muted-foreground">System Settings</p>
        <p className="text-sm">Configure how VoiceFlow integrates with your system.</p>
        <p className="text-sm text-muted-foreground">
          Audio upload format (WAV / MP3) moved to Presets — each preset carries its own, so you can
          send MP3 to a metered provider and WAV to a local one.
        </p>
      </div>

      <div className="flex items-center justify-between rounded-lg border p-4">
        <div className="space-y-0.5">
          <Label htmlFor="autostart" className="text-base">
            Launch at Startup
          </Label>
          <p className="text-sm text-muted-foreground">
            Automatically start VoiceFlow when you log in to your computer
          </p>
        </div>
        <Switch
          id="autostart"
          checked={isEnabled ?? false}
          onCheckedChange={handleToggleAutostart}
          disabled={isCheckingStatus || isToggling}
        />
      </div>

      <div className="flex items-center justify-between rounded-lg border p-4">
        <div className="space-y-0.5">
          <Label className="text-base">Keep History For</Label>
          <p className="text-sm text-muted-foreground mr-6">
            Transcribed text is stored locally so you can search and re-paste it. Audio is never
            saved. Older entries are deleted automatically.
          </p>
        </div>
        <div className="flex shrink-0 items-center gap-1 rounded-lg border bg-accent/50 p-1">
          {RETENTION_OPTIONS.map((option) => (
            <Button
              key={option.days}
              variant={appConfig?.historyRetentionDays === option.days ? 'default' : 'ghost'}
              size="sm"
              onClick={() => handleRetentionChange(option.days)}
              disabled={saveAppConfig.isPending}
              className="h-8 px-3 text-xs font-medium"
            >
              {option.label}
            </Button>
          ))}
        </div>
      </div>

      <div className="flex items-center justify-between rounded-lg border p-4">
        <div className="space-y-0.5">
          <Label className="text-base">Restart Onboarding</Label>
          <p className="text-sm text-muted-foreground">
            Go through the initial setup wizard again. This can help troubleshoot configuration
            issues or reset your preferences
          </p>
        </div>
        <Button
          variant="outline"
          size="sm"
          onClick={() => restartOnboarding.mutate()}
          disabled={restartOnboarding.isPending}
        >
          <RotateCcw className="mr-2 h-4 w-4" />
          {restartOnboarding.isPending ? 'Restarting...' : 'Restart'}
        </Button>
      </div>
    </div>
  )
}
