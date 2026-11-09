import type { ReactNode } from 'react'
import { faceSrc, type Face } from './mascot.ts'

type Props = {
  title: string
  description?: string
  /** 다음 행동(버튼·링크) */
  action?: ReactNode
  /** 기본은 졸린 표정(빈 화면) */
  face?: Face
}

export default function EmptyState({ title, description, action, face = 'sleepy' }: Props) {
  return (
    <div className='flex flex-col items-center gap-3 px-4 py-12 text-center'>
      <img src={faceSrc(face)} alt='' className='size-32' />
      <h2 className='text-lg font-bold'>{title}</h2>
      {description && <p className='max-w-sm text-sm text-text-muted'>{description}</p>}
      {action && <div className='mt-2'>{action}</div>}
    </div>
  )
}
