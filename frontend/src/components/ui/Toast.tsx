import Icon from './Icon.tsx'

export type ToastData = {
  id: number
  /** success=라임 아이콘·강조선, info·error=흰색(에러에도 빨강을 쓰지 않는다) */
  tone: 'success' | 'info' | 'error'
  message: string
  /** 서버 오류일 때 Grafana 추적용 */
  requestId?: string
}

export function Toast({ toast, onClose }: { toast: ToastData; onClose: () => void }) {
  const success = toast.tone === 'success'
  return (
    <div
      role={toast.tone === 'error' ? 'alert' : 'status'}
      // 왼쪽 3px 강조선 = 성공은 라임, 그 외 흰색
      className={`chamfer pointer-events-auto flex w-full items-center gap-2.5 bg-surface-2 px-3.5 py-3 [--c:6px] ${success ? 'shadow-[inset_3px_0_0_var(--color-brand)]' : 'shadow-[inset_3px_0_0_var(--color-text)]'}`}
    >
      <Icon name={success ? 'check' : toast.tone === 'error' ? 'alert' : 'info'} className={`size-[18px] ${success ? 'text-brand' : 'text-text'}`} />
      <p className='min-w-0 grow text-sm'>{toast.message}</p>
      {toast.requestId && (
        <code className='font-mono text-[11px] text-text-muted'>
          <span className='sr-only'>request_id </span>
          {toast.requestId}
        </code>
      )}
      <button type='button' onClick={onClose} aria-label='알림 닫기' className='text-text-muted hover:text-text'>
        <Icon name='close' className='size-4' />
      </button>
    </div>
  )
}

/** 데스크톱은 오른쪽 아래, 모바일은 위에 쌓이는 토스트 목록. 상태는 ToastProvider가 쥔다. */
export function ToastViewport({ toasts, onClose }: { toasts: ToastData[]; onClose: (id: number) => void }) {
  return (
    <div className='pointer-events-none fixed top-4 right-4 z-50 flex w-[min(24rem,calc(100vw-2rem))] flex-col gap-2 sm:top-auto sm:bottom-4'>
      {toasts.map((t) => (
        <Toast key={t.id} toast={t} onClose={() => onClose(t.id)} />
      ))}
    </div>
  )
}
