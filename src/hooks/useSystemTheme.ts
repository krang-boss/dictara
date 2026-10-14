import { useEffect } from 'react'

/**
 * Windows that paint no surface of their own and so cannot be themed.
 *
 * The recording popup floats free over whatever is behind it; giving it a
 * background — light or dark — would put a rectangle around the orb.
 */
const UNTHEMED_WINDOWS = ['recording-popup']

/**
 * Follow the system appearance.
 *
 * Every window that has a surface — preferences, onboarding and the menu bar
 * panel — uses the same tokens and the same source of truth, so the app never
 * shows a light panel next to a dark one. The initial class is set by the
 * inline script in `index.html` to avoid a flash; this only keeps it in sync
 * when the user switches appearance while the app is running.
 */
export function useSystemTheme() {
  useEffect(() => {
    // Set by the inline script in index.html, from the first path segment —
    // available synchronously, unlike the Tauri window label.
    const windowPath = document.documentElement.dataset.windowPath ?? ''

    if (UNTHEMED_WINDOWS.includes(windowPath)) {
      document.documentElement.classList.remove('dark')
      return
    }

    const darkModeQuery = window.matchMedia('(prefers-color-scheme: dark)')

    const applyDarkMode = (isDark: boolean) => {
      document.documentElement.classList.toggle('dark', isDark)
    }

    applyDarkMode(darkModeQuery.matches)

    const handleChange = (event: MediaQueryListEvent) => applyDarkMode(event.matches)
    darkModeQuery.addEventListener('change', handleChange)

    return () => darkModeQuery.removeEventListener('change', handleChange)
  }, [])
}
