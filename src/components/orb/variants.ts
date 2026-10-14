import type { PopupStyle } from '@/bindings'

/**
 * Starting points for the laboratory.
 *
 * Each variant is a complete style — picking one and then moving the sliders is
 * the intended workflow, so these are tuned to be good defaults rather than
 * extremes. The `variant` id is stored alongside the tweaked values so the UI
 * can still show which one a custom style grew out of.
 *
 * The palette is a ramp rather than a set: the first colour is the densest
 * sky, the last is the lit cloud bank. Four stops is the natural length —
 * fewer still works, it just flattens the sky.
 */
export interface OrbVariant {
  id: string
  name: string
  description: string
  style: Omit<PopupStyle, 'variant' | 'showTimer'>
}

const base = {
  size: 200,
} as const

export const VARIANTS: OrbVariant[] = [
  {
    id: 'periwinkle',
    name: 'Periwinkle',
    description: 'Blue-violet sky with a milk-white bank. The original.',
    style: {
      ...base,
      palette: ['#5c63fb', '#7a8ffb', '#b8c7f9', '#e3ebfe'],
      speed: 0.38,
      turbulence: 0.5,
      softness: 0.08,
      saturation: 0.64,
      reactivity: 0.55,
    },
  },
  {
    id: 'dusk',
    name: 'Dusk',
    description: 'Violet deepening into rose. The warmest of the set.',
    style: {
      ...base,
      palette: ['#3b1f5e', '#7c3aed', '#e879b9', '#fde8f3'],
      speed: 0.32,
      turbulence: 0.55,
      softness: 0.12,
      saturation: 0.68,
      reactivity: 0.55,
    },
  },
  {
    id: 'mint',
    name: 'Mint',
    description: 'Sea-green weather. Cool, quiet, low contrast.',
    style: {
      ...base,
      palette: ['#0f4d46', '#14b8a6', '#7ee7d5', '#eafcf7'],
      speed: 0.4,
      turbulence: 0.45,
      softness: 0.1,
      saturation: 0.6,
      reactivity: 0.6,
    },
  },
  {
    id: 'kiln',
    name: 'Kiln',
    description: 'Fire under smoke. Dark base, bright bank, fast weather.',
    style: {
      ...base,
      palette: ['#5b1a08', '#ea580c', '#fbbf5f', '#fff3dd'],
      speed: 0.5,
      turbulence: 0.62,
      softness: 0.1,
      saturation: 0.72,
      reactivity: 0.7,
    },
  },
  {
    id: 'overcast',
    name: 'Overcast',
    description: 'Near-monochrome. The most discreet — sits on any desktop.',
    style: {
      ...base,
      palette: ['#1c1e26', '#4b5262', '#9aa3b5', '#f1f4f9'],
      speed: 0.3,
      turbulence: 0.5,
      softness: 0.14,
      saturation: 0.35,
      reactivity: 0.5,
    },
  },
]

/** Turn a variant into a complete style, preserving user-level preferences. */
export function styleFromVariant(
  variant: OrbVariant,
  carryOver?: Pick<PopupStyle, 'size' | 'showTimer'>
): PopupStyle {
  return {
    ...variant.style,
    variant: variant.id,
    size: carryOver?.size ?? variant.style.size,
    showTimer: carryOver?.showTimer ?? true,
  }
}

export function findVariant(id: string): OrbVariant | undefined {
  return VARIANTS.find((variant) => variant.id === id)
}
