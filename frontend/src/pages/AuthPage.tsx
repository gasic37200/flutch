import { Navigate, useSearchParams } from 'react-router'
import AuthLayout from '../features/auth/components/AuthLayout.tsx'
import KakaoSignupForm from '../features/auth/components/KakaoSignupForm.tsx'
import LoginForm from '../features/auth/components/LoginForm.tsx'
import SignupForm from '../features/auth/components/SignupForm.tsx'
import { safeNext, useAuth } from '../features/auth/hooks/useAuth.ts'

type Props = { mode: 'login' | 'signup' | 'kakao' }

const loginHero = (
  <>
    <img src='/brand/logo-flutch.png' alt='FLUTCH' className='h-12 self-start' />
    <img src='/brand/mascot-controller.png' alt='컨트롤러를 든 FLUTCH 마스코트 고양이' className='w-[min(300px,60%)] self-center' />
    <div className='flex flex-col gap-2'>
      <span className='text-3xl leading-tight font-black'>
        심장이 뛰는 순간,
        <br />
        플러치.
      </span>
      <span className='font-display font-bold tracking-wide text-brand italic'>Feel the Moment. Live on FLUTCH.</span>
    </div>
  </>
)

const signupHero = (
  <>
    <img src='/brand/logo-flutch.png' alt='FLUTCH' className='h-11 self-start' />
    <div className='flex items-center gap-4'>
      <img src='/brand/face-joy.png' alt='' className='size-22 rounded-full' />
      <span className='text-2xl leading-snug font-black'>
        가입하면 바로
        <br />
        방송하고 응원할 수 있어요.
      </span>
    </div>
  </>
)

export default function AuthPage({ mode }: Props) {
  const { me } = useAuth()
  const [params] = useSearchParams()
  // 로그인 성공(me 갱신)하면 가려던 곳(next)으로 돌아간다. 이미 로그인한 채 들어와도 마찬가지.
  if (me) return <Navigate to={safeNext(params.get('next'))} replace />
  return (
    <AuthLayout hero={mode === 'login' ? loginHero : signupHero}>
      <h1 className='mb-5 text-2xl font-black sm:text-[26px]'>{{ login: '로그인', signup: '회원가입', kakao: '닉네임 설정' }[mode]}</h1>
      {mode === 'login' ? <LoginForm /> : mode === 'signup' ? <SignupForm /> : <KakaoSignupForm />}
    </AuthLayout>
  )
}
