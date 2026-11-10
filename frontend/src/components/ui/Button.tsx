import type { ButtonHTMLAttributes, ReactNode } from 'react'
import { Link } from 'react-router'

// 빨강(danger)은 "방송 종료"에만(DESIGN §2). outline은 영구 밴·삭제·탈퇴 확인용 흰 테두리.
const variants = {
  primary: { edge: 'bg-brand', face: 'bg-brand text-bg hover:brightness-110' },
  secondary: { edge: 'bg-brand-line hover:bg-brand', face: 'bg-surface-2 text-text' },
  danger: { edge: 'bg-live', face: 'bg-live text-text hover:brightness-110' },
  outline: { edge: 'bg-text', face: 'bg-surface-2 text-text' },
}
const sizes = { lg: 'h-12 px-5 text-base', md: 'h-10 px-4 text-sm', sm: 'h-8 px-3 text-xs', icon: 'size-10' }

type Props = Omit<ButtonHTMLAttributes<HTMLButtonElement>, 'children'> & {
  variant?: keyof typeof variants
  size?: keyof typeof sizes
  /** 로딩 중에는 비활성 + 중복 클릭 차단 */
  loading?: boolean
  /** 주면 링크(<a>)로 렌더 */
  to?: string
  children: ReactNode
}

export default function Button({ variant = 'primary', size = 'md', loading, to, className = '', children, disabled, type = 'button', ...rest }: Props) {
  const v = variants[variant]
  const edge = `chamfer inline-flex grow p-px [--c:6px] ${v.edge} disabled:cursor-not-allowed disabled:opacity-50`
  const content = (
    <span className={`chamfer-inner inline-flex grow items-center justify-center gap-2 font-semibold whitespace-nowrap ${sizes[size]} ${v.face}`}>
      {loading && <span className='size-4 animate-spin rounded-full border-2 border-current border-t-transparent motion-reduce:animate-none' />}
      {children}
    </span>
  )
  return (
    <span className={`inline-flex has-[:focus-visible]:drop-shadow-focus ${className}`}>
      {to ? (
        <Link to={to} className={edge}>
          {content}
        </Link>
      ) : (
        <button {...rest} type={type} disabled={disabled || loading} aria-busy={loading || undefined} className={edge}>
          {content}
        </button>
      )}
    </span>
  )
}
