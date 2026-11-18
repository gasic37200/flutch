import { Outlet } from 'react-router'
import { ToastProvider } from '../../../components/ui/ToastProvider.tsx'
import AppShell from '../../../layouts/AppShell.tsx'
import { useSidebarPanel } from '../../follow/hooks/useFollow.ts'
import { AuthContext, useLogoutToHome, useMe } from '../hooks/useAuth.ts'

// 로그인 정보를 Context로 내려 주고 AppShell(헤더)에 user·onLogout을 넘기는 얇은 래퍼. bare면 셸 없이(로그인·가입 전체 화면).
export default function AuthShell({ bare }: { bare?: boolean }) {
  const { data, isPending } = useMe()
  const me = data ?? null
  const logout = useLogoutToHome()
  const panel = useSidebarPanel(me)
  return (
    <AuthContext value={{ me, isPending }}>
      {bare ? (
        <ToastProvider>
          <Outlet />
        </ToastProvider>
      ) : (
        <AppShell panel={panel} user={me && { id: me.id, nickname: me.nickname, balance: me.balance }} onLogout={logout} />
      )}
    </AuthContext>
  )
}
