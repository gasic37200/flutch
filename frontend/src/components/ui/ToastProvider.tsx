import { useCallback, useState, type ReactNode } from 'react'
import { ToastContext, type ShowToast } from '../../hooks/useToast.ts'
import { ToastViewport, type ToastData } from './Toast.tsx'

let nextId = 1

/** 성공·안내는 4초 뒤 사라지고, 에러는 사용자가 닫을 때까지 남는다(DESIGN §9). */
export function ToastProvider({ children }: { children: ReactNode }) {
  const [toasts, setToasts] = useState<ToastData[]>([])
  const close = useCallback((id: number) => setToasts((all) => all.filter((t) => t.id !== id)), [])
  const show = useCallback<ShowToast>(
    (toast) => {
      const id = nextId++
      setToasts((all) => [...all, { ...toast, id }])
      if (toast.tone !== 'error') setTimeout(() => close(id), 4000)
    },
    [close],
  )
  return (
    <ToastContext value={show}>
      {children}
      <ToastViewport toasts={toasts} onClose={close} />
    </ToastContext>
  )
}
