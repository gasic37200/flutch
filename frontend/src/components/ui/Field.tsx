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

// 기본·포커스·에러·비활성 상태를 테두리로 구분한다(포커스·에러 = 밝은 테두리, 비활성 = 흐리게).
export default function Field(props: InputProps | AreaProps) {
  const { label, hideLabel, hint, error, counter, className = '', ...rest } = props
  const id = useId()
  const noteId = `${id}-note`
  const note = error ?? hint
  const edge = error ? 'bg-text' : 'bg-brand-line focus-within:bg-brand'
  const control = `chamfer-inner block w-full bg-surface-2 px-3 text-sm text-text placeholder:text-text-dim outline-none disabled:opacity-50`
  const shared = { id, 'aria-invalid': error ? true : undefined, 'aria-describedby': note ? noteId : undefined }
  const length = counter && typeof rest.value === 'string' ? rest.value.length : null

  return (
    <div className={className}>
      <label htmlFor={id} className={hideLabel ? 'sr-only' : 'mb-1 block text-sm font-semibold text-text-muted'}>
        {label}
      </label>
      <div className={`chamfer p-px [--c:6px] ${edge}`}>
        {props.multiline ? (
          <textarea {...(rest as TextareaHTMLAttributes<HTMLTextAreaElement>)} {...shared} className={`${control} min-h-24 py-2`} />
        ) : (
          <input {...(rest as InputHTMLAttributes<HTMLInputElement>)} {...shared} className={`${control} h-10`} />
        )}
      </div>
      {(note || length !== null) && (
        <div className='mt-1 flex items-start justify-between gap-2 text-xs'>
          <p id={noteId} className={`flex items-center gap-1 ${error ? 'text-text' : 'text-text-dim'}`}>
            {error && <Icon name='alert' className='size-3.5 text-brand' />}
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
