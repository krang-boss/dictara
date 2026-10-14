import { createFileRoute } from '@tanstack/react-router'
import { Lab } from '@/components/lab'

export const Route = createFileRoute('/preferences/popup')({
  component: PopupLabRoute,
})

function PopupLabRoute() {
  return <Lab />
}
