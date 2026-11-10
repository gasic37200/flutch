import { useId, type InputHTMLAttributes, type TextareaHTMLAttributes } from 'react'
import Icon from './Icon.tsx'

type Common = {
  label: string
  /** 레이블을 시각적으로 숨김(검색창·채팅창). 스크린리더에는 남는다. */
  hideLabel?: boolean
  hint?: string
  /** 에러 문구. 빨강 대신 흰 글자 + 아이콘 */
  error?: string
  /** 글자 수 표시(maxLength와 value가 있을 때) */
  counter?: boolean
  className?: string
}
type InputProps = Common & { multiline?: false } & InputHTMLAttributes<HTMLInputElement>
type AreaProps = Common & { multiline: true } & TextareaHTMLAttributes<HTMLTextAreaElement>

// 기본 = 은은한 라임 테두리, 포커스 = 라임 + 글로우, 에러 = 흰 테두리 + 흰 글자·아이콘(빨강 금지), 비활성 = 테두리 없이 어두운 배경.
export default function Field(props: InputProps | AreaProps) {
  const { label, hideLabel, hint, error, counter, className = '', ...rest } = props
  const id = useId()
  const noteId = `${id}-note`
  const note = error ?? hint
  const edge = rest.disabled ? 'bg-surface-dim' : error ? 'bg-text' : 'bg-brand-line focus-within:bg-brand'
  const control = `chamfer-inner block w-full bg-surface px-3 text-sm text-text placeholder:text-text-dim outline-none disabled:bg-surface-dim disabled:text-text-dim`
  const shared = { id, 'aria-invalid': error ? true : undefined, 'aria-describedby': note ? noteId : undefined }
  const length = counter && typeof rest.value === 'string' ? rest.value.length : null

  return (
    <div className={className}>
      <label htmlFor={id} className={hideLabel ? 'sr-only' : 'mb-1 block text-sm font-semibold text-text-muted'}>
        {label}
      </label>
      {/* 글로우는 clip-path 밖 래퍼에 건다 */}
      <div className='focus-within:drop-shadow-field'>
        <div className={`chamfer p-px [--c:6px] ${edge}`}>
          {props.multiline ? (
            <textarea {...(rest as TextareaHTMLAttributes<HTMLTextAreaElement>)} {...shared} className={`${control} min-h-24 py-2`} />
          ) : (
            <input {...(rest as InputHTMLAttributes<HTMLInputElement>)} {...shared} className={`${control} h-10`} />
          )}
        </div>
      </div>
      {(note || length !== null) && (
        <div className='mt-1 flex items-start justify-between gap-2 text-xs'>
          <p id={noteId} className={`flex items-center gap-1 ${error ? 'text-text' : 'text-text-dim'}`}>
            {error && <Icon name='alert' className='size-3.5 text-text' />}
            {note}
          </p>
          {length !== null && (
            <span className='font-mono text-text-dim'>
              {length}/{rest.maxLength}
            </span>
          )}
        </div>
      )}
    </div>
  )
}
