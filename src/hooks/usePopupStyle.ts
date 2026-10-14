import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { commands, type PopupStyle } from '@/bindings'

export const POPUP_STYLE_QUERY_KEY = ['popupStyle'] as const

function unwrap<T>(result: { status: 'ok'; data: T } | { status: 'error'; error: string }): T {
  if (result.status === 'error') throw new Error(result.error)
  return result.data
}

/** The recording popup's appearance. */
export function usePopupStyle() {
  return useQuery({
    queryKey: POPUP_STYLE_QUERY_KEY,
    queryFn: async (): Promise<PopupStyle> => unwrap(await commands.loadPopupStyle()),
  })
}

function usePopupStyleMutation<TArgs>(mutationFn: (args: TArgs) => Promise<PopupStyle>) {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn,
    onSuccess: (style) => {
      // The command returns the sanitized style, so seed the cache with what
      // was actually stored rather than what we asked for.
      queryClient.setQueryData(POPUP_STYLE_QUERY_KEY, style)
    },
  })
}

export function useSavePopupStyle() {
  return usePopupStyleMutation(async (style: PopupStyle) =>
    unwrap(await commands.savePopupStyle(style))
  )
}

export function useResetPopupStyle() {
  return usePopupStyleMutation(async () => unwrap(await commands.resetPopupStyle()))
}
