import { Navigate, useSearchParams } from 'react-router'
import Panel from '../components/ui/Panel.tsx'
import LoginForm from '../features/auth/components/LoginForm.tsx'
import SignupForm from '../features/auth/components/SignupForm.tsx'
import { safeNext, useAuth } from '../features/auth/hooks/useAuth.ts'

type Props = { mode: 'login' | 'signup' }

export default function AuthPage({ mode }: Props) {
  const { me } = useAuth()
  const [params] = useSearchParams()
  // 로그인 성공(me 갱신)하면 가려던 곳(next)으로 돌아간다. 이미 로그인한 채 들어와도 마찬가지.
  if (me) return <Navigate to={safeNext(params.get('next'))} replace />
  return (
    <div className='mx-auto max-w-md'>
      <Panel innerClassName='p-6'>
        <h1 className='mb-6 font-display text-2xl font-bold tracking-wide text-text'>{mode === 'login' ? '로그인' : '회원가입'}</h1>
        {mode === 'login' ? <LoginForm /> : <SignupForm />}
      </Panel>
    </div>
  )
}
