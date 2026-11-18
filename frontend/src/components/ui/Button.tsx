import type { ButtonHTMLAttributes, ReactNode } from 'react'
import { Link } from 'react-router'

// 크기(바깥 높이): xl 50 = 인증 폼 주 버튼, lg 44 = 기본·대표 CTA·터치 최소, md 40 = 패널 안 보조, sm 36 = 밀집 목록·인라인.
// 빨강(danger)은 "방송 종료"에만(DESIGN §2), burst(노랑)는 SUPER BURST에만. outline은 영구 밴·삭제·탈퇴 확인용 흰 테두리.
// neutral은 테두리 없는 일반 버튼(취소·아이콘). 테두리 층(edge)과 안쪽 층(face)을 따로 칠해 1px 테두리를 만든다.
const variants = {
  primary: { edge: 'bg-brand', face: 'bg-brand text-bg font-bold hover:bg-brand-hover' },
  secondary: { edge: 'bg-brand', face: 'bg-surface text-brand font-bold hover:bg-surface-2' },
  neutral: { edge: 'bg-surface-2', face: 'bg-surface-2 text-text font-medium hover:brightness-125' },
  danger: { edge: 'bg-live', face: 'bg-live text-text font-bold hover:brightness-110' },
  outline: { edge: 'bg-text', face: 'bg-surface text-text font-bold' },
  burst: { edge: 'bg-burst', face: 'bg-burst text-bg font-display font-bold italic tracking-wide hover:brightness-110' },
}
const sizes = { xl: 'h-12 px-6 text-base', lg: 'h-[42px] px-5 text-[15px]', md: 'h-[38px] px-4 text-sm', sm: 'h-[34px] px-4 text-sm', icon: 'size-[42px]' }

type Props = Omit<ButtonHTMLAttributes<HTMLButtonElement>, 'children'> & {
  variant?: keyof typeof variants
  size?: keyof typeof sizes
  /** 로딩 중에는 비활성 + 중복 클릭 차단 */
  loading?: boolean
  /** 주면 링크(<a>)로 렌더 */
  to?: string
  children: ReactNode
}

export default function Button({ variant = 'primary', size = 'lg', loading, to, className = '', children, disabled, type = 'button', ...rest }: Props) {
  const v = variants[variant]
  // 비활성(로딩 제외)은 회색 칠 + 흐린 글자. 로딩은 원래 색을 유지한다.
  const off = disabled && !loading
  const edge = `chamfer inline-flex grow p-px [--c:6px] ${off ? 'bg-border' : v.edge} disabled:cursor-not-allowed`
  const face = off ? 'bg-border text-text-dim font-bold' : v.face
  const content = (
    <span className={`chamfer-inner inline-flex grow items-center justify-center gap-2 whitespace-nowrap ${sizes[size]} ${face}`}>
      {loading && <span className='size-4 animate-spin rounded-full border-2 border-current border-t-transparent motion-reduce:animate-none' />}
      {children}
    </span>
  )
  return (
    <span className={`inline-flex has-[:focus-visible]:drop-shadow-focus ${className}`}>
      {to ? (
        <Link to={to} aria-label={rest['aria-label']} className={edge}>
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
