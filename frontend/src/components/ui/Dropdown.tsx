import { useEffect, useRef, useState, type ReactNode } from 'react'
import { Link } from 'react-router'
import Panel from './Panel.tsx'

/** muted = 흐린 글자 + 위 구분선(로그아웃처럼 메뉴 맨 아래 항목) */
export type DropdownItem = { label: string; to?: string; onSelect?: () => void; muted?: boolean }

type Props = {
  /** 트리거 접근성 이름(아바타 버튼처럼 글자가 없을 때 필요) */
  label: string
  trigger: ReactNode
  items: DropdownItem[]
  /** 메뉴 위에 붙는 영역(닉네임·잔액 등). 아래 구분선은 이 컴포넌트가 그린다. */
  header?: ReactNode
  align?: 'left' | 'right'
}

// shortcut: 방향키 이동 없이 Tab·ESC·바깥 클릭만 지원, 메뉴 항목이 많아지면 방향키 추가
export default function Dropdown({ label, trigger, items, header, align = 'right' }: Props) {
  const [open, setOpen] = useState(false)
  const root = useRef<HTMLDivElement>(null)

  useEffect(() => {
    if (!open) return
    const onPointer = (e: PointerEvent) => root.current?.contains(e.target as Node) || setOpen(false)
    const onKey = (e: KeyboardEvent) => e.key === 'Escape' && setOpen(false)
    document.addEventListener('pointerdown', onPointer)
    document.addEventListener('keydown', onKey)
    return () => {
      document.removeEventListener('pointerdown', onPointer)
      document.removeEventListener('keydown', onKey)
    }
  }, [open])

  const itemClass = (muted?: boolean) => `block w-full px-3 py-2.5 text-left text-sm hover:bg-surface-2 ${muted ? 'border-t border-surface-2 text-text-muted' : 'text-text'}`
  return (
    <div ref={root} className='relative'>
      <button type='button' aria-label={label} aria-haspopup='menu' aria-expanded={open} onClick={() => setOpen(!open)} className='flex size-11 items-center justify-center'>
        {trigger}
      </button>
      {open && (
        <Panel className={`absolute top-full z-40 mt-1.5 w-60 ${align === 'right' ? 'right-0' : 'left-0'}`} innerClassName='p-2'>
          {header && <div className='border-b border-surface-2 px-3 py-2.5'>{header}</div>}
          <div role='menu'>
            {items.map((item) =>
              item.to ? (
                <Link key={item.label} to={item.to} role='menuitem' onClick={() => setOpen(false)} className={itemClass(item.muted)}>
                  {item.label}
                </Link>
              ) : (
                <button
                  key={item.label}
                  type='button'
                  role='menuitem'
                  onClick={() => {
                    setOpen(false)
                    item.onSelect?.()
                  }}
                  className={itemClass(item.muted)}
                >
                  {item.label}
                </button>
              ),
            )}
          </div>
        </Panel>
      )}
    </div>
  )
}
