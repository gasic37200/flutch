import type { ReactNode } from 'react'
import { Link } from 'react-router'

// 스튜디오 전용 상단바(시안): 사이드바·검색 없이 로고 · STUDIO · 상태 · 오른쪽 버튼만.
export default function StudioTopBar({ children }: { children?: ReactNode }) {
  return (
    <header className='border-b border-surface-2'>
      <div className='mx-auto flex min-h-16 max-w-[1920px] flex-wrap items-center gap-x-4 gap-y-3 px-4 py-2.5 md:px-6 md:py-0'>
        <Link to='/' aria-label='FLUTCH 홈'>
          <img src='/brand/logo-flutch.png' alt='FLUTCH' className='block h-7 w-auto' />
        </Link>
        <span className='hidden font-display font-bold tracking-widest text-text-muted md:inline'>STUDIO</span>
        {children}
      </div>
    </header>
  )
}
