import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { commands, type HistoryEntry, type HistoryQuery, type HistoryStats } from '@/bindings'

export const HISTORY_QUERY_KEY = ['history'] as const
export const HISTORY_STATS_QUERY_KEY = ['historyStats'] as const

function unwrap<T>(result: { status: 'ok'; data: T } | { status: 'error'; error: string }): T {
  if (result.status === 'error') throw new Error(result.error)
  return result.data
}

/** Transcription history, newest first. */
export function useHistory(query?: Partial<HistoryQuery>) {
  const resolved: HistoryQuery = {
    search: query?.search ?? null,
    presetId: query?.presetId ?? null,
    limit: query?.limit ?? 50,
    offset: query?.offset ?? 0,
  }

  return useQuery({
    queryKey: [...HISTORY_QUERY_KEY, resolved],
    queryFn: async (): Promise<HistoryEntry[]> => unwrap(await commands.listHistory(resolved)),
  })
}

export function useHistoryStats() {
  return useQuery({
    queryKey: HISTORY_STATS_QUERY_KEY,
    queryFn: async (): Promise<HistoryStats> => unwrap(await commands.historyStats()),
  })
}

function useHistoryMutation<TArgs>(mutationFn: (args: TArgs) => Promise<unknown>) {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: HISTORY_QUERY_KEY })
      queryClient.invalidateQueries({ queryKey: HISTORY_STATS_QUERY_KEY })
    },
  })
}

export function useDeleteHistoryEntry() {
  return useHistoryMutation(async (id: number) => unwrap(await commands.deleteHistoryEntry(id)))
}

export function useClearHistory() {
  return useHistoryMutation(async () => unwrap(await commands.clearHistory()))
}

export function usePruneHistory() {
  return useHistoryMutation(async () => unwrap(await commands.pruneHistory()))
}

/** Paste text back into whichever app is frontmost. */
export function usePasteText() {
  return useMutation({
    mutationFn: async (text: string) => unwrap(await commands.pasteText(text)),
  })
}
