import { StepContainer } from '../StepContainer'
import { useOnboardingNavigation } from '@/hooks/useOnboardingNavigation'
import { usePresets } from '@/hooks/usePresets'
import { Presets } from '@/components/presets'

/**
 * Onboarding: set up the first preset.
 *
 * Reuses the real presets screen rather than a parallel simplified one — there
 * is only one correct way to configure a provider, and duplicating it here is
 * how the two drift apart.
 */
export function ApiKeysStep() {
  const { goNext, goBack, skipOnboarding, isNavigating } = useOnboardingNavigation()
  const { data: config, isLoading } = usePresets()

  const hasActivePreset = !!config?.activeId

  if (isLoading) {
    return (
      <StepContainer
        title="Transcription"
        description="Loading…"
        showBack={true}
        showSkip={true}
        onBack={() => goBack('api_keys')}
        onSkip={() => skipOnboarding.mutate()}
      >
        <div className="text-muted-foreground">Loading configuration…</div>
      </StepContainer>
    )
  }

  return (
    <StepContainer
      title="Transcription"
      description="Create a preset: a provider, a model, and the options to go with it."
      onNext={() => hasActivePreset && goNext('api_keys')}
      nextDisabled={!hasActivePreset}
      onBack={() => goBack('api_keys')}
      onSkip={() => skipOnboarding.mutate()}
      isLoading={isNavigating || skipOnboarding.isPending}
    >
      <Presets />

      {hasActivePreset && (
        <p className="mt-4 text-sm text-emerald-500">Ready. Continue to set up your shortcuts.</p>
      )}
    </StepContainer>
  )
}
