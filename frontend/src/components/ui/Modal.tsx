import { useId, type ReactNode } from 'react'
import Icon from './Icon.tsx'
import Panel from './Panel.tsx'

type Props = {
  open: boolean
  onClose: () => void
  title: string
  children: ReactNode
  /** 하단 버튼 영역 */
  footer?: ReactNode
}

// 네이티브 <dialog>로 ESC·포커스 가두기를 얻는다. 열릴 때 첫 입력창에 포커스, 배경 클릭으로 닫는다.
export default function Modal({ open, onClose, title, children, footer }: Props) {
  const titleId = useId()
  if (!open) return null
  return (
    <dialog
      ref={(el) => {
        if (!el || el.open) return
        el.showModal()
        el.querySelector<HTMLElement>('input, textarea, select')?.focus()
      }}
      aria-labelledby={titleId}
      onClose={onClose}
      // Panel이 dialog 전체를 채우므로 dialog 자신이 클릭 대상이면 배경이다
      onClick={(e) => e.target === e.currentTarget && onClose()}
      className='m-auto bg-transparent p-0 text-text backdrop:bg-bg/80'
    >
      <Panel className='w-[min(28rem,calc(100vw-2rem))]' innerClassName='space-y-4 p-5'>
        <div className='flex items-center justify-between gap-4'>
          <h2 id={titleId} className='text-lg font-bold'>
            {title}
          </h2>
          <button type='button' onClick={onClose} aria-label='닫기' className='text-text-muted hover:text-text'>
            <Icon name='close' />
          </button>
        </div>
        <div>{children}</div>
        {footer && <div className='flex justify-end gap-2'>{footer}</div>}
      </Panel>
    </dialog>
  )
}
