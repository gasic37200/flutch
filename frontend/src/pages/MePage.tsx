import AccountSettings from '../features/auth/components/AccountSettings.tsx'
import { useRequireLogin } from '../features/auth/hooks/useAuth.ts'

export default function MePage() {
  const { me } = useRequireLogin()
  if (!me) return null
  return (
    <div className='max-w-[420px]'>
      <h1 className='mb-4 text-2xl font-black sm:text-[26px]'>내 정보</h1>
      <AccountSettings me={me} />
    </div>
  )
}
