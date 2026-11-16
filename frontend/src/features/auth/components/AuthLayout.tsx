import type { ReactNode } from 'react'

// 로그인·가입 공통: lg 이상 2열(브랜드 패널 + 폼, 최대 폭·세로 가운데), 그 아래는 작은 브랜드 띠 + 1열 폼. 보드는 셸 없는 전체 화면이지만 셸(헤더·사이드바) 안에서 그린다.
export default function AuthLayout({ hero, children }: { hero: ReactNode; children: ReactNode }) {
  return (
    <div className='mx-auto grid w-full max-w-[1120px] gap-6 lg:min-h-[calc(100dvh-10rem)] lg:grid-cols-[minmax(0,1fr)_minmax(380px,440px)] lg:items-center lg:gap-12'>
      <section
        className='chamfer relative flex items-center justify-between gap-4 overflow-hidden bg-surface bg-cover bg-center px-5 py-4 [--c:8px] lg:hidden'
        style={{ backgroundImage: 'url(/brand/bg-speed.jpg)' }}
      >
        <div className='absolute inset-0 bg-linear-to-r from-bg/20 to-bg/75' />
        <div className='relative flex flex-col gap-2'>
          <img src='/brand/logo-flutch.png' alt='FLUTCH' className='h-8 self-start' />
          <span className='font-display text-sm font-bold tracking-wide text-brand italic'>Feel the Moment. Live on FLUTCH.</span>
        </div>
        <img src='/brand/mascot-controller.png' alt='' className='relative h-24 w-auto shrink-0 object-contain' />
      </section>
      <section
        className='chamfer relative hidden min-h-[520px] flex-col overflow-hidden bg-surface bg-cover bg-center p-10 [--c:12px] lg:flex'
        style={{ backgroundImage: 'url(/brand/bg-speed.jpg)' }}
      >
        <div className='absolute inset-0 bg-linear-to-r from-bg/20 to-bg/75' />
        <div className='relative flex grow flex-col justify-between gap-6'>{hero}</div>
      </section>
      <section className='w-full max-w-[440px] py-2 lg:py-6'>{children}</section>
    </div>
  )
}
