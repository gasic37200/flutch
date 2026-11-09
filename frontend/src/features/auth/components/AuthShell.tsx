import AppShell from '../../../layouts/AppShell.tsx'
import { usePopularChannels } from '../../broadcast/hooks/useBroadcasts.ts'
import { AuthContext, useLogoutToHome, useMe } from '../hooks/useAuth.ts'

// 로그인 정보를 Context로 내려 주고 AppShell(헤더)에 user·onLogout을 넘기는 얇은 래퍼.
export default function AuthShell() {
  const { data, isPending } = useMe()
  const me = data ?? null
  const logout = useLogoutToHome()
  const channels = usePopularChannels()
  return (
    <AuthContext value={{ me, isPending }}>
      <AppShell channels={channels} user={me && { id: me.id, nickname: me.nickname, balance: me.balance }} onLogout={logout} />
    </AuthContext>
  )
}
