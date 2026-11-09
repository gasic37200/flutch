import AccountSettings from '../features/auth/components/AccountSettings.tsx'
import { useLogoutToHome, useRequireLogin } from '../features/auth/hooks/useAuth.ts'

export default function MePage() {
  const { me } = useRequireLogin()
  const logout = useLogoutToHome()
  if (!me) return null
  return (
    <div className='mx-auto max-w-md'>
      <h1 className='mb-4 font-display text-2xl font-bold tracking-wide text-text'>내 정보</h1>
      <AccountSettings me={me} onLogout={logout} />
    </div>
  )
}
