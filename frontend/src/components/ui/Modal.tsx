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
  /** 라임 테두리로 강조(방송 이어하기처럼 지금 눌러야 하는 창) */
  emphasis?: boolean
}

// 네이티브 <dialog>로 ESC·포커스 가두기를 얻는다. 열릴 때 첫 입력창에 포커스, 배경 클릭으로 닫는다.
export default function Modal({ open, onClose, title, children, footer, emphasis }: Props) {
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
      className='m-auto bg-transparent p-0 text-text backdrop:bg-bg/70'
    >
      <Panel selected={emphasis} className='w-[min(28rem,calc(100vw-2rem))]' innerClassName='flex flex-col gap-3.5 p-5'>
        <div className='flex items-center'>
          <h2 id={titleId} className='text-[17px] font-bold'>
            {title}
          </h2>
          <button type='button' onClick={onClose} aria-label='닫기' className='-my-2 -mr-2 ml-auto flex size-10 items-center justify-center text-text-muted hover:text-text'>
            <Icon name='close' className='size-[18px]' />
          </button>
        </div>
        <div className='text-sm'>{children}</div>
        {footer && <div className='flex justify-end gap-2'>{footer}</div>}
      </Panel>
    </dialog>
  )
}
