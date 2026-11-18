import { Link } from 'react-router'
import Button from '../../../components/ui/Button.tsx'
import Field from '../../../components/ui/Field.tsx'
import { useKakaoSignupForm } from '../hooks/useAuthForms.ts'
import { AuthCard, tall } from './AuthLayout.tsx'
import { CheckRow, FormAlert } from './FormParts.tsx'

const link = 'text-xs text-brand hover:underline'

/** 카카오 첫 로그인 직후: 가입은 여기서 닉네임·약관을 확인해야 끝난다. */
export default function KakaoSignupForm() {
  const f = useKakaoSignupForm()
  if (f.loading) return null
  if (f.expired) {
    return (
      <AuthCard title='닉네임 설정' compact>
        <FormAlert message='카카오 로그인 시간이 지났어요. 처음부터 다시 시도해 주세요.' />
        <Button to='/login' size='xl' className='w-full [&_.chamfer-inner]:h-[46px]'>
          로그인으로 돌아가기
        </Button>
      </AuthCard>
    )
  }
  return (
    <AuthCard title='FLUTCH에서 쓸 닉네임을 정해 주세요' description='카카오 닉네임을 미리 채워 뒀어요. 중복·금지어 검사를 거쳐요.' compact>
      <form onSubmit={f.submit} className='flex flex-col gap-3.5'>
        <Field className={`${tall} [&_input]:h-11`} label='닉네임' hint='2~12자 한글·영문·숫자, 바꾸면 30일 뒤에 다시 바꿀 수 있어요' value={f.values.nickname} onChange={(e) => f.set.nickname(e.target.value)} error={f.error?.fields.nickname} counter maxLength={12} required />
        <div className='chamfer flex flex-col gap-2 bg-surface-2 px-3.5 py-3 text-[13px] [--c:6px] [&_label]:text-[13px] [&_label]:text-text'>
          <CheckRow compact checked={f.values.terms} onChange={f.set.terms}>
            [필수] 이용약관·개인정보처리방침 동의
          </CheckRow>
          {/* 시안엔 없지만 약관을 읽고 동의하게 하려고 둔 링크(decisions §4) */}
          <span className='flex gap-3 pl-[30px]'>
            <Link to='/terms' target='_blank' className={link}>
              이용약관 보기
            </Link>
            <Link to='/privacy' target='_blank' className={link}>
              개인정보처리방침 보기
            </Link>
          </span>
          <CheckRow compact checked={f.values.over14} onChange={f.set.over14}>
            [필수] 만 14세 이상이에요
          </CheckRow>
        </div>
        <FormAlert message={f.error?.message} />
        <Button type='submit' size='xl' loading={f.pending} disabled={!f.canSubmit} className='w-full [&_.chamfer-inner]:h-[46px]'>
          시작하기
        </Button>
      </form>
    </AuthCard>
  )
}
