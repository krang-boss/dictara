import { createFileRoute } from '@tanstack/react-router'
import { Presets } from '@/components/presets'

export const Route = createFileRoute('/preferences/presets')({
  component: PresetsRoute,
})

function PresetsRoute() {
  return <Presets />
}
