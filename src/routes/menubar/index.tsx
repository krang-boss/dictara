import { createFileRoute } from '@tanstack/react-router'
import { MenuBarPanel } from '@/components/menubar'

export const Route = createFileRoute('/menubar/')({
  component: MenuBarRoute,
})

function MenuBarRoute() {
  return <MenuBarPanel />
}
