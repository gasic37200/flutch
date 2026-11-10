import type { ReactNode } from 'react'
import Panel from './Panel.tsx'
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
    <Panel innerClassName='flex flex-col items-center gap-2.5 p-6 text-center'>
      <img src={faceSrc(face)} alt='' className='size-18 rounded-full' />
      <h2 className='text-[15px] font-bold'>{title}</h2>
      {description && <p className='max-w-sm text-[13px] text-text-muted'>{description}</p>}
      {action && <div className='mt-1'>{action}</div>}
    </Panel>
  )
}
