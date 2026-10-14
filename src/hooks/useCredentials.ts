import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { commands, type CredentialStatus, type Provider } from '@/bindings'

export const CREDENTIALS_QUERY_KEY = ['credentials'] as const

function unwrap<T>(result: { status: 'ok'; data: T } | { status: 'error'; error: string }): T {
  if (result.status === 'error') throw new Error(result.error)
  return result.data
}

/**
 * Which providers have an API key stored.
 *
 * Keys themselves are never returned — only whether one exists.
 */
export function useCredentials() {
  return useQuery({
    queryKey: CREDENTIALS_QUERY_KEY,
    queryFn: async (): Promise<CredentialStatus[]> => unwrap(await commands.listCredentials()),
  })
}

export function useSaveCredentials() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async (params: { provider: Provider; apiKey: string }) =>
      unwrap(await commands.saveCredentials(params.provider, params.apiKey)),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: CREDENTIALS_QUERY_KEY })
    },
  })
}

export function useDeleteCredentials() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async (provider: Provider) => unwrap(await commands.deleteCredentials(provider)),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: CREDENTIALS_QUERY_KEY })
    },
  })
}
