import type { SuperClutch } from '../../../api/generated/SuperClutch.ts'

// 3초 전체 화면 연출(소리 없음). 동작 줄임 설정이면 정적 배너만 보여 준다.
export default function SuperClutchOverlay({ superClutch }: { superClutch: SuperClutch | undefined }) {
  if (!superClutch) return null
  return (
    <div
      role='status'
      aria-label='SUPER CLUTCH 발동'
      className='pointer-events-none fixed inset-0 z-50 flex items-center justify-center bg-bg/80 motion-safe:animate-super-in'
    >
      <div className='absolute inset-0 bg-clutch opacity-20 motion-safe:animate-beat' />
      <img src='/brand/logo-super-clutch.png' alt='SUPER CLUTCH' className='relative h-40 w-auto object-contain md:h-64' />
    </div>
  )
}
