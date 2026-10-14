import { useMutation, useQueryClient } from '@tanstack/react-query'
import { commands, type RecordingTrigger } from '@/bindings'

interface SaveAppConfigParams {
  recordingTrigger?: RecordingTrigger
  historyRetentionDays?: number
}

export function useSaveAppConfig() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async (params: SaveAppConfigParams): Promise<void> => {
      const result = await commands.saveAppConfig(
        params.recordingTrigger ?? null,
        params.historyRetentionDays ?? null
      )
      if (result.status === 'error') {
        throw new Error(result.error)
      }
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['appConfig'] })
    },
  })
}
