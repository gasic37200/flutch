import type { ReactNode } from 'react'

// CSS만으로 호버·키보드 포커스에 뜬다. 자식 버튼에는 aria-label을 따로 준다(툴팁은 보조).
// 헤더처럼 화면 위쪽에 붙은 버튼도 잘리지 않게 아래로 연다.
export default function Tooltip({ text, children }: { text: string; children: ReactNode }) {
  return (
    <span className='group relative inline-flex'>
      {children}
      <span
        role='tooltip'
        className='chamfer pointer-events-none invisible absolute top-full left-1/2 z-40 mt-2 -translate-x-1/2 bg-text px-2.5 py-1.5 text-xs font-bold whitespace-nowrap text-bg opacity-0 transition-opacity [--c:6px] group-focus-within:visible group-focus-within:opacity-100 group-hover:visible group-hover:opacity-100'
      >
        {text}
      </span>
    </span>
  )
}
