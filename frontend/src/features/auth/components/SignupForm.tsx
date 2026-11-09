import { Link } from 'react-router'
import Button from '../../../components/ui/Button.tsx'
import Field from '../../../components/ui/Field.tsx'
import { useSignupForm } from '../hooks/useAuthForms.ts'
import { CheckRow, FormAlert } from './FormParts.tsx'

const link = 'font-semibold text-brand hover:underline'

export default function SignupForm() {
  const f = useSignupForm()
  return (
    <form onSubmit={f.submit} className='space-y-4'>
      <Field label='아이디' hint='영문 소문자·숫자 4~20자 (가입 후 변경 불가)' value={f.values.loginId} onChange={(e) => f.set.loginId(e.target.value)} error={f.error?.fields.loginId} autoComplete='username' autoCapitalize='none' maxLength={20} required />
      <Field label='비밀번호' type='password' hint='영문+숫자 8~64자' value={f.values.password} onChange={(e) => f.set.password(e.target.value)} error={f.error?.fields.password} autoComplete='new-password' maxLength={64} required />
      <Field label='닉네임' hint='한글·영문·숫자 2~12자' value={f.values.nickname} onChange={(e) => f.set.nickname(e.target.value)} error={f.error?.fields.nickname} counter maxLength={12} required />
      <div className='space-y-2 border-t border-border pt-4'>
        <CheckRow checked={f.values.agreeTerms} onChange={f.set.agreeTerms}>
          <Link to='/terms' target='_blank' className={link}>
            이용약관
          </Link>
          에 동의합니다 (필수)
        </CheckRow>
        <CheckRow checked={f.values.agreePrivacy} onChange={f.set.agreePrivacy}>
          <Link to='/privacy' target='_blank' className={link}>
            개인정보처리방침
          </Link>
          에 동의합니다 (필수)
        </CheckRow>
        <CheckRow checked={f.values.over14} onChange={f.set.over14}>
          만 14세 이상입니다 (필수)
        </CheckRow>
      </div>
      <FormAlert message={f.error?.message} />
      <Button type='submit' loading={f.pending} disabled={!f.canSubmit} className='w-full'>
        가입하기
      </Button>
      <p className='text-center text-sm text-text-muted'>
        이미 계정이 있나요?{' '}
        <Link to='/login' className={link}>
          로그인
        </Link>
      </p>
    </form>
  )
}
