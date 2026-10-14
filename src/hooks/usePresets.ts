import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { commands, type Preset, type PresetsConfig, type ProviderInfo } from '@/bindings'

export const PRESETS_QUERY_KEY = ['presets'] as const
export const PROVIDERS_QUERY_KEY = ['providers'] as const

function unwrap<T>(result: { status: 'ok'; data: T } | { status: 'error'; error: string }): T {
  if (result.status === 'error') throw new Error(result.error)
  return result.data
}

/** All presets plus which one is active. */
export function usePresets() {
  return useQuery({
    queryKey: PRESETS_QUERY_KEY,
    queryFn: async (): Promise<PresetsConfig> => unwrap(await commands.loadPresets()),
  })
}

/** Static per-provider requirements, used to drive the preset editor. */
export function useProviders() {
  return useQuery({
    queryKey: PROVIDERS_QUERY_KEY,
    queryFn: async (): Promise<ProviderInfo[]> => commands.listProviders(),
    // These never change at runtime.
    staleTime: Infinity,
  })
}

function usePresetMutation<TArgs>(mutationFn: (args: TArgs) => Promise<unknown>) {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: PRESETS_QUERY_KEY })
    },
  })
}

/** Create or update a preset. */
export function useSavePreset() {
  return usePresetMutation(async (preset: Preset) => unwrap(await commands.savePreset(preset)))
}

export function useDuplicatePreset() {
  return usePresetMutation(async (id: string) => unwrap(await commands.duplicatePreset(id)))
}

export function useDeletePreset() {
  return usePresetMutation(async (id: string) => unwrap(await commands.deletePreset(id)))
}

export function useSetActivePreset() {
  return usePresetMutation(async (id: string | null) => unwrap(await commands.setActivePreset(id)))
}

export function useCycleActivePreset() {
  return usePresetMutation(async () => unwrap(await commands.cycleActivePreset()))
}

export function useReorderPresets() {
  return usePresetMutation(async (ids: string[]) => unwrap(await commands.reorderPresets(ids)))
}

/**
 * Run a preset against a silent clip.
 *
 * Resolves to `false` when the credentials are rejected, and rejects when the
 * request itself failed — the two are meaningfully different to the user.
 */
export function useTestPreset() {
  return useMutation({
    mutationFn: async (preset: Preset): Promise<boolean> =>
      unwrap(await commands.testPreset(preset)),
  })
}
