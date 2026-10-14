import { createFileRoute } from '@tanstack/react-router'
import { History } from '@/components/history'

export const Route = createFileRoute('/preferences/history')({
  component: HistoryRoute,
})

function HistoryRoute() {
  return <History />
}
