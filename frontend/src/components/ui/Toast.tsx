import Icon from './Icon.tsx'
import Panel from './Panel.tsx'

export type ToastData = {
  id: number
  /** success=라임 아이콘, info·error=흰색(에러에도 빨강을 쓰지 않는다) */
  tone: 'success' | 'info' | 'error'
  message: string
  /** 서버 오류일 때 Grafana 추적용 */
  requestId?: string
}

export function Toast({ toast, onClose }: { toast: ToastData; onClose: () => void }) {
  return (
    <Panel size='sm' role={toast.tone === 'error' ? 'alert' : 'status'} className='pointer-events-auto w-full' innerClassName='flex items-start gap-3 bg-surface-2 p-3'>
      <Icon name={toast.tone === 'success' ? 'check' : toast.tone === 'error' ? 'alert' : 'info'} className={`mt-0.5 size-5 ${toast.tone === 'success' ? 'text-brand' : 'text-text'}`} />
      <div className='min-w-0 grow text-sm'>
        <p>{toast.message}</p>
        {toast.requestId && <p className='mt-1 font-mono text-xs text-text-dim'>request_id: {toast.requestId}</p>}
      </div>
      <button type='button' onClick={onClose} aria-label='알림 닫기' className='text-text-muted hover:text-text'>
        <Icon name='close' className='size-4' />
      </button>
    </Panel>
  )
}

/** 화면 우하단에 쌓이는 토스트 목록. 상태는 ToastProvider가 쥔다. */
export function ToastViewport({ toasts, onClose }: { toasts: ToastData[]; onClose: (id: number) => void }) {
  return (
    <div className='pointer-events-none fixed right-4 bottom-4 z-50 flex w-[min(24rem,calc(100vw-2rem))] flex-col gap-2'>
      {toasts.map((t) => (
        <Toast key={t.id} toast={t} onClose={() => onClose(t.id)} />
      ))}
    </div>
  )
}
