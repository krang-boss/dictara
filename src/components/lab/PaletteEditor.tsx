import { Plus, X } from 'lucide-react'
import { Label } from '@/components/ui/label'

const MAX_COLORS = 6

interface PaletteEditorProps {
  palette: string[]
  onChange: (palette: string[]) => void
}

/**
 * Colour stops for the orb.
 *
 * The shader reads the palette as a ramp from the densest sky to the lit cloud
 * bank, so order is meaning rather than preference — which is why the stops are
 * laid out left to right with the ramp itself shown underneath.
 */
export function PaletteEditor({ palette, onChange }: PaletteEditorProps) {
  const setColor = (index: number, color: string) => {
    const next = [...palette]
    next[index] = color
    onChange(next)
  }

  const removeColor = (index: number) => {
    // Two stops is the minimum the ramp can interpolate between.
    if (palette.length <= 2) return
    onChange(palette.filter((_, i) => i !== index))
  }

  const addColor = () => {
    if (palette.length >= MAX_COLORS) return
    onChange([...palette, palette[palette.length - 1] ?? '#ffffff'])
  }

  return (
    <section className="space-y-2">
      <div className="flex items-baseline justify-between">
        <Label className="text-xs uppercase tracking-wide text-muted-foreground">Palette</Label>
        <span className="text-xs text-muted-foreground/70">
          Darkest sky first, lit bank last · {palette.length}/{MAX_COLORS}
        </span>
      </div>

      <div className="flex flex-wrap items-center gap-2">
        {palette.map((color, index) => (
          <div key={index} className="group relative">
            <input
              type="color"
              value={color.slice(0, 7)}
              onChange={(e) => setColor(index, e.target.value)}
              className="h-11 w-11 cursor-pointer rounded-lg border border-border/60 bg-transparent p-0"
              aria-label={`Colour ${index + 1}`}
            />
            {palette.length > 2 && (
              <button
                type="button"
                onClick={() => removeColor(index)}
                className="absolute -right-1 -top-1 hidden h-4 w-4 items-center justify-center rounded-full bg-destructive text-destructive-foreground group-hover:flex"
                aria-label={`Remove colour ${index + 1}`}
              >
                <X className="h-2.5 w-2.5" />
              </button>
            )}
          </div>
        ))}

        {palette.length < MAX_COLORS && (
          <button
            type="button"
            onClick={addColor}
            className="flex h-11 w-11 items-center justify-center rounded-lg border border-dashed border-border/60 text-muted-foreground transition-colors hover:border-border hover:text-foreground"
            aria-label="Add colour"
          >
            <Plus className="h-4 w-4" />
          </button>
        )}
      </div>

      <div
        className="h-2 rounded-full"
        style={{ background: `linear-gradient(to right, ${palette.join(', ')})` }}
      />
    </section>
  )
}
