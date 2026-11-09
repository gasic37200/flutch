import { Link } from 'react-router'
import Button from '../../../components/ui/Button.tsx'
import Field from '../../../components/ui/Field.tsx'
import { useLoginForm } from '../hooks/useAuthForms.ts'
import { CheckRow, FormAlert } from './FormParts.tsx'

const TEST_ACCOUNTS = [
  { account: 'streamer1', label: '스트리머1' },
  { account: 'streamer2', label: '스트리머2' },
  { account: 'viewer', label: '시청자' },
] as const

export default function LoginForm() {
  const f = useLoginForm()
  return (
    <div className='space-y-6'>
      <form onSubmit={f.submit} className='space-y-4'>
        <Field label='아이디' value={f.values.loginId} onChange={(e) => f.set.loginId(e.target.value)} error={f.error?.fields.loginId} autoComplete='username' autoCapitalize='none' required />
        <Field label='비밀번호' type='password' value={f.values.password} onChange={(e) => f.set.password(e.target.value)} error={f.error?.fields.password} autoComplete='current-password' required />
        <CheckRow checked={f.values.remember} onChange={f.set.remember}>
          로그인 상태 유지 (30일)
        </CheckRow>
        <FormAlert message={f.error?.message} />
        <Button type='submit' loading={f.pending} className='w-full'>
          로그인
        </Button>
        <p className='text-center text-sm text-text-muted'>
          아직 계정이 없나요?{' '}
          <Link to='/signup' className='font-semibold text-brand hover:underline'>
            회원가입
          </Link>
        </p>
      </form>
      <section aria-labelledby='test-login' className='space-y-2 border-t border-border pt-4'>
        <h2 id='test-login' className='text-sm font-semibold text-text-muted'>
          테스트 계정으로 바로 로그인
        </h2>
        <div className='grid grid-cols-3 gap-2'>
          {TEST_ACCOUNTS.map(({ account, label }) => (
            <Button key={account} variant='secondary' size='sm' disabled={f.pending} onClick={() => f.testLogin(account)}>
              {label}
            </Button>
          ))}
        </div>
      </section>
    </div>
  )
}
