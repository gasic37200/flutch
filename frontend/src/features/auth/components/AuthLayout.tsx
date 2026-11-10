import type { ReactNode } from 'react'

// 로그인·가입 공통 2단: 왼쪽 브랜드 패널(md 이상) + 오른쪽 폼. 보드는 셸 없는 전체 화면이지만 셸(헤더·사이드바) 안에서 그린다.
export default function AuthLayout({ hero, children }: { hero: ReactNode; children: ReactNode }) {
  return (
    <div className='flex flex-wrap gap-6 lg:gap-10'>
      <section
        className='chamfer relative hidden min-h-[460px] grow basis-[420px] flex-col overflow-hidden bg-surface bg-cover bg-center p-10 [--c:12px] md:flex'
        style={{ backgroundImage: 'url(/brand/bg-speed.jpg)' }}
      >
        <div className='absolute inset-0 bg-linear-to-r from-bg/20 to-bg/75' />
        <div className='relative flex grow flex-col justify-between gap-6'>{hero}</div>
      </section>
      <section className='flex grow basis-[400px] justify-center py-2 md:py-6'>
        <div className='w-full max-w-[420px]'>{children}</div>
      </section>
    </div>
  )
}
