import { useEffect, useRef, useState } from 'react'
import { Outlet } from 'react-router'
import Icon from '../components/ui/Icon.tsx'
import { ToastProvider } from '../components/ui/ToastProvider.tsx'
import Header, { type ShellUser } from './Header.tsx'
import Sidebar, { type SidebarPanel } from './Sidebar.tsx'

type Props = {
  /** 로그인 정보(없으면 비로그인 헤더). 인증 task가 Context에서 읽어 넘긴다. */
  user?: ShellUser | null
  /** 사이드바 "인기 라이브" 목록 */
  panel?: SidebarPanel
  onLogout?: () => void
}

// 헤더 + 왼쪽 사이드바 + 본문(Outlet). lg 미만에서는 사이드바가 햄버거로 여는 서랍이 된다.
export default function AppShell({ user = null, panel, onLogout }: Props) {
  const [drawer, setDrawer] = useState(false)
  const closeRef = useRef<HTMLButtonElement>(null)

  useEffect(() => {
    if (!drawer) return
    closeRef.current?.focus()
    const onKey = (e: KeyboardEvent) => e.key === 'Escape' && setDrawer(false)
    document.addEventListener('keydown', onKey)
    return () => document.removeEventListener('keydown', onKey)
  }, [drawer])

  return (
    <ToastProvider>
      <a href='#main' className='sr-only focus:not-sr-only focus:absolute focus:z-50 focus:bg-surface-2 focus:p-2'>
        본문으로 건너뛰기
      </a>
      <Header user={user} onMenu={() => setDrawer(true)} onLogout={onLogout} />
      <div className='flex'>
        <div className='sticky top-14 hidden h-[calc(100dvh-3.5rem)] w-60 shrink-0 border-r border-surface-2 md:top-16 md:h-[calc(100dvh-4rem)] lg:block'>
          <Sidebar panel={panel} loggedIn={!!user} />
        </div>
        {drawer && (
          <div role='dialog' aria-modal='true' aria-label='메뉴' className='fixed inset-0 z-40 lg:hidden'>
            <button type='button' tabIndex={-1} aria-hidden onClick={() => setDrawer(false)} className='absolute inset-0 bg-bg/80' />
            <div className='relative h-full w-64 max-w-[80vw] border-r border-surface-2'>
              <Sidebar panel={panel} loggedIn={!!user} onNavigate={() => setDrawer(false)} />
              <button ref={closeRef} type='button' aria-label='메뉴 닫기' onClick={() => setDrawer(false)} className='absolute top-2 right-2 flex size-11 items-center justify-center text-text-muted hover:text-text'>
                <Icon name='close' />
              </button>
            </div>
          </div>
        )}
        <main id='main' className='min-w-0 grow p-4 lg:p-6'>
          <Outlet />
        </main>
      </div>
    </ToastProvider>
  )
}
