import type { ReactNode } from 'react'
import Icon from '../../../components/ui/Icon.tsx'

/** 폼 전체에 대한 에러. 빨강 대신 흰 글자 + 아이콘(DESIGN). */
export function FormAlert({ message }: { message?: string }) {
  if (!message) return null
  return (
    <p role='alert' className='flex items-start gap-2 text-sm text-text'>
      <Icon name='alert' className='mt-0.5 size-4 shrink-0 text-brand' />
      {message}
    </p>
  )
}

export function CheckRow({ checked, onChange, children }: { checked: boolean; onChange: (v: boolean) => void; children: ReactNode }) {
  return (
    <label className='flex items-center gap-2 text-sm text-text'>
      <input type='checkbox' checked={checked} onChange={(e) => onChange(e.target.checked)} className='size-4 accent-brand' />
      <span>{children}</span>
    </label>
  )
}
