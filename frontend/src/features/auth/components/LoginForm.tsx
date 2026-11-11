import { Link } from 'react-router'
import Button from '../../../components/ui/Button.tsx'
import Field from '../../../components/ui/Field.tsx'
import Panel from '../../../components/ui/Panel.tsx'
import { faceSrc } from '../../../components/ui/mascot.ts'
import { useLoginForm } from '../hooks/useAuthForms.ts'
import { CheckRow, FormAlert } from './FormParts.tsx'

const TEST_ACCOUNTS = [
  { account: 'streamer1', label: '스트리머1', face: 'excited', ring: 'border-ring-3' },
  { account: 'streamer2', label: '스트리머2', face: 'joy', ring: 'border-ring-4' },
  { account: 'viewer', label: '시청자', face: 'happy', ring: 'border-ring-5' },
] as const

export default function LoginForm() {
  const f = useLoginForm()
  return (
    <div className='space-y-5'>
      <form onSubmit={f.submit} className='space-y-5'>
        <Field label='아이디' value={f.values.loginId} onChange={(e) => f.set.loginId(e.target.value)} error={f.error?.fields.loginId} autoComplete='username' autoCapitalize='none' required />
        <Field label='비밀번호' type='password' value={f.values.password} onChange={(e) => f.set.password(e.target.value)} error={f.error?.fields.password} autoComplete='current-password' required />
        <CheckRow checked={f.values.remember} onChange={f.set.remember}>
          로그인 상태 유지 (30일)
        </CheckRow>
        <FormAlert message={f.error?.message ?? (f.kakaoFailed ? '카카오 로그인에 실패했어요. 다시 시도해 주세요.' : undefined)} />
        <Button type='submit' size='lg' loading={f.pending} className='w-full'>
          로그인
        </Button>
        <p className='text-[13px] text-text-muted'>
          아직 계정이 없나요?{' '}
          <Link to='/signup' className='inline-flex min-h-11 items-center font-semibold text-brand hover:underline'>
            회원가입
          </Link>{' '}
          · 비밀번호를 잊으면 복구할 수 없어요.
        </p>
      </form>
      {f.kakaoEnabled && (
        // 카카오 인증은 서버가 리다이렉트하는 페이지 이동이라 <Link>가 아니라 일반 링크
        <span className='flex has-[:focus-visible]:drop-shadow-focus'>
          <a href='/api/v1/auth/kakao/start' className='chamfer flex h-[42px] grow items-center justify-center bg-surface-2 text-[15px] font-bold [--c:6px] hover:brightness-125'>
            카카오로 로그인
          </a>
        </span>
      )}
      <div className='flex items-center gap-3 text-[13px] text-text-dim'>
        <span className='h-px grow bg-surface-2' />
        심사·체험용
        <span className='h-px grow bg-surface-2' />
      </div>
      <Panel innerClassName='flex min-w-0 flex-col gap-2.5 p-4'>
        <section aria-labelledby='test-login' className='flex flex-col gap-2.5'>
          <h2 id='test-login' className='text-sm font-bold'>
            테스트 계정으로 바로 로그인
          </h2>
          <div className='grid grid-cols-3 gap-2'>
            {TEST_ACCOUNTS.map(({ account, label, face, ring }) => (
              <span key={account} className='flex min-w-0 has-[:focus-visible]:drop-shadow-focus'>
                <button
                  type='button'
                  disabled={f.pending}
                  onClick={() => f.testLogin(account)}
                  className='chamfer flex h-[72px] grow flex-col items-center justify-center gap-1.5 bg-surface-2 text-[13px] [--c:6px] hover:brightness-125 disabled:cursor-not-allowed disabled:opacity-60'
                >
                  <img src={faceSrc(face)} alt='' className={`size-7 rounded-full border-2 ${ring}`} />
                  {label}
                </button>
              </span>
            ))}
          </div>
          <p className='text-xs text-text-dim'>스트리머1로 방송을 만들고, 스트리머2로 수정·삭제를 시도하면 권한 거절을 확인할 수 있어요.</p>
        </section>
      </Panel>
    </div>
  )
}
