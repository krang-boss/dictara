import { commands } from '@/bindings'
import { Link, useRouterState } from '@tanstack/react-router'
import { error as logError } from '@tauri-apps/plugin-log'
import { Clock, Keyboard, Settings, Sparkles, Waves } from 'lucide-react'
import { useEffect, useState, type ReactNode } from 'react'
import { Separator } from '../ui/separator'
import {
  Sidebar,
  SidebarContent,
  SidebarGroup,
  SidebarGroupContent,
  SidebarInset,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarProvider,
  SidebarTrigger,
} from '../ui/sidebar'

interface PreferencesLayoutProps {
  children: ReactNode
}

const menuItems = [
  {
    title: 'Presets',
    url: '/preferences/presets',
    icon: Sparkles,
  },
  {
    title: 'History',
    url: '/preferences/history',
    icon: Clock,
  },
  {
    title: 'Popup',
    url: '/preferences/popup',
    icon: Waves,
  },
  {
    title: 'Shortcuts',
    url: '/preferences/hotkeys',
    icon: Keyboard,
  },
  {
    title: 'System',
    url: '/preferences/system',
    icon: Settings,
  },
]

export function PreferencesLayout({ children }: PreferencesLayoutProps) {
  const routerState = useRouterState()
  const currentPath = routerState.location.pathname
  const [sidebarOpen, setSidebarOpen] = useState(true)
  const [appVersion, setAppVersion] = useState<string | null>(null)

  // Get current page title
  const currentPage = menuItems.find((item) => item.url === currentPath)
  const pageTitle = currentPage?.title ?? 'Preferences'

  // Fetch app version
  useEffect(() => {
    commands
      .getAppVersion()
      .then((v) => setAppVersion(v))
      .catch((e: unknown) => {
        logError(`[PreferencesLayout] Failed to load app version: ${e}`)
      })
  }, [])

  return (
    <SidebarProvider
      open={sidebarOpen}
      onOpenChange={setSidebarOpen}
      className="h-screen"
      style={{ '--sidebar-width': '14rem' } as React.CSSProperties}
    >
      <Sidebar collapsible="icon">
        <SidebarContent>
          <SidebarGroup>
            <SidebarGroupContent>
              <SidebarMenu>
                {menuItems.map((item) => (
                  <SidebarMenuItem key={item.title}>
                    <SidebarMenuButton asChild isActive={currentPath === item.url}>
                      <Link to={item.url}>
                        <item.icon />
                        <span>{item.title}</span>
                      </Link>
                    </SidebarMenuButton>
                  </SidebarMenuItem>
                ))}
              </SidebarMenu>
            </SidebarGroupContent>
          </SidebarGroup>
        </SidebarContent>
      </Sidebar>
      <SidebarInset className="flex h-full flex-col overflow-hidden">
        <header className="flex h-14 shrink-0 items-center gap-2 border-b px-4">
          <SidebarTrigger className="-ml-1" />
          <Separator orientation="vertical" className="mr-2 h-4" />
          <span className="font-medium">{pageTitle}</span>
        </header>
        <main className="flex-1 overflow-y-auto overflow-x-hidden p-6">{children}</main>
        <footer className="flex h-8 shrink-0 items-center justify-end border-t bg-sidebar px-4 text-xs text-sidebar-foreground/60">
          <span>{appVersion ? `VoiceFlow ${appVersion}` : ''}</span>
        </footer>
      </SidebarInset>
    </SidebarProvider>
  )
}
