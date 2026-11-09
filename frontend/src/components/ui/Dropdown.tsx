import { useEffect, useRef, useState, type ReactNode } from 'react'
import { Link } from 'react-router'
import Panel from './Panel.tsx'

export type DropdownItem = { label: string; to?: string; onSelect?: () => void }

type Props = {
  /** 트리거 접근성 이름(아바타 버튼처럼 글자가 없을 때 필요) */
  label: string
  trigger: ReactNode
  items: DropdownItem[]
  /** 메뉴 위에 붙는 영역(모바일 잔액 등) */
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

  const itemClass = 'block w-full px-4 py-2 text-left text-sm text-text hover:bg-surface-2'
  return (
    <div ref={root} className='relative'>
      <button type='button' aria-label={label} aria-haspopup='menu' aria-expanded={open} onClick={() => setOpen(!open)} className='flex items-center'>
        {trigger}
      </button>
      {open && (
        <Panel size='sm' className={`absolute top-full z-40 mt-2 min-w-44 ${align === 'right' ? 'right-0' : 'left-0'}`} innerClassName='py-1'>
          {header}
          <div role='menu'>
            {items.map((item) =>
              item.to ? (
                <Link key={item.label} to={item.to} role='menuitem' onClick={() => setOpen(false)} className={itemClass}>
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
                  className={itemClass}
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
