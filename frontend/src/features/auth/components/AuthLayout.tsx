import type { ReactNode } from 'react'
import { Link } from 'react-router'
import Panel from '../../../components/ui/Panel.tsx'

const shade = 'radial-gradient(ellipse at 50% 40%, color-mix(in srgb, var(--color-bg) 72%, transparent) 0%, color-mix(in srgb, var(--color-bg) 90%, transparent) 55%, color-mix(in srgb, var(--color-bg) 97%, transparent) 100%)'

// 시안(Auth·Signup): 셸 없는 전체 화면. 배경 텍스처 + 가장자리 어두운 막, 가운데에 로고·슬로건·카드 한 줄. 600px 이하(컨테이너 기준)는 위쪽 정렬·여백 축소.
export default function AuthLayout({ children }: { children: ReactNode }) {
  return (
    <div className='@container relative min-h-dvh bg-bg bg-cover bg-center' style={{ backgroundImage: 'url(/brand/bg-speed.jpg)' }}>
      <div className='absolute inset-0' style={{ background: shade }} aria-hidden />
      <main className='relative mx-auto flex min-h-dvh flex-col items-center justify-center gap-6 px-6 pt-14 pb-16 @max-[600px]:justify-start @max-[600px]:gap-[18px] @max-[600px]:px-4 @max-[600px]:pt-7 @max-[600px]:pb-10'>
        <div className='flex flex-col items-center gap-2.5 text-center'>
          <Link to='/' aria-label='FLUTCH 홈'>
            <img src='/brand/logo-flutch.png' alt='FLUTCH' className='h-11 @max-[600px]:h-8' />
          </Link>
          <span className='font-display text-[15px] font-bold tracking-[.04em] text-brand italic @max-[600px]:text-[13px]'>Feel the Moment. Live on FLUTCH.</span>
        </div>
        {children}
      </main>
    </div>
  )
}

/** 폼 카드(로그인 440 · 가입 480). */
export function AuthCard({ title, description, wide, children }: { title: string; description?: string; wide?: boolean; children: ReactNode }) {
  return (
    <Panel className={`w-full ${wide ? 'max-w-[480px]' : 'max-w-[440px]'}`} innerClassName='flex flex-col gap-[18px] bg-surface/95 p-8 @max-[600px]:p-5'>
      <div className='flex flex-col gap-1'>
        <h1 className='text-[26px] font-black @max-[600px]:text-[22px]'>{title}</h1>
        {description && <span className='text-[13px] text-text-muted'>{description}</span>}
      </div>
      {children}
    </Panel>
  )
}

/** 입력칸을 시안 높이(48)로 키운다. */
export const tall = '[&_input]:h-[46px] [&_input]:text-[15px]'
