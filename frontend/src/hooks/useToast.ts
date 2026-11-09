import { createContext, useContext } from 'react'
import type { ToastData } from '../components/ui/Toast.tsx'

export type ShowToast = (toast: Omit<ToastData, 'id'>) => void

export const ToastContext = createContext<ShowToast | null>(null)

/** 토스트 띄우기: `useToast()({ tone: 'success', message: '저장했어요' })`. AppShell 안에서만 동작. */
export function useToast(): ShowToast {
  const show = useContext(ToastContext)
  if (!show) throw new Error('useToast는 ToastProvider 안에서만 쓸 수 있습니다')
  return show
}
