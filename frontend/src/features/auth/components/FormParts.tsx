import type { ReactNode } from 'react'
import Icon from '../../../components/ui/Icon.tsx'

/** 폼 전체에 대한 에러. 빨강 대신 흰 글자 + 아이콘(DESIGN). */
export function FormAlert({ message }: { message?: string }) {
  if (!message) return null
  return (
    <p role='alert' className='flex items-start gap-2 text-sm text-text-muted'>
      <Icon name='alert' className='mt-0.5 size-4 shrink-0 text-brand' />
      {message}
    </p>
  )
}

/** compact: 박스 안에 모아 둔 줄(시안 약관 박스)은 44px 높이 대신 글자 높이 */
export function CheckRow({ checked, onChange, children, compact }: { checked: boolean; onChange: (v: boolean) => void; children: ReactNode; compact?: boolean }) {
  return (
    <label className={`flex items-center gap-2 ${compact ? 'min-h-6' : 'min-h-11'} text-sm text-text-muted`}>
      <input type='checkbox' checked={checked} onChange={(e) => onChange(e.target.checked)} className='ml-1 mr-[3px] size-[18px] accent-brand' />
      <span>{children}</span>
    </label>
  )
}
