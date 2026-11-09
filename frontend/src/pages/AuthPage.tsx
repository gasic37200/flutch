type Props = { mode: 'login' | 'signup' }

export default function AuthPage({ mode }: Props) {
  return (
    <main>
      <h1>{mode === 'login' ? '로그인' : '회원가입'}</h1>
    </main>
  )
}
