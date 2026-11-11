import { Link } from 'react-router'
import Button from '../../../components/ui/Button.tsx'
import Field from '../../../components/ui/Field.tsx'
import Panel from '../../../components/ui/Panel.tsx'
import { useSignupForm } from '../hooks/useAuthForms.ts'
import { CheckRow, FormAlert } from './FormParts.tsx'

const link = 'inline-flex min-h-11 items-center px-1 text-[13px] text-brand hover:underline'

export default function SignupForm() {
  const f = useSignupForm()
  return (
    <form onSubmit={f.submit} className='space-y-[18px]'>
      <Field label='아이디' hint='4~20자 영문 소문자·숫자, 나중에 바꿀 수 없어요' value={f.values.loginId} onChange={(e) => f.set.loginId(e.target.value)} error={f.error?.fields.loginId} autoComplete='username' autoCapitalize='none' maxLength={20} required />
      <Field label='비밀번호' type='password' hint='영문+숫자 8~64자' value={f.values.password} onChange={(e) => f.set.password(e.target.value)} error={f.error?.fields.password} autoComplete='new-password' maxLength={64} required />
      <Field label='닉네임' hint='2~12자 한글·영문·숫자, 바꾸면 30일 뒤에 다시 바꿀 수 있어요' value={f.values.nickname} onChange={(e) => f.set.nickname(e.target.value)} error={f.error?.fields.nickname} counter maxLength={12} required />
      <Panel innerClassName='flex flex-col gap-2.5 px-4 py-3.5'>
        <label className='flex min-h-11 items-center gap-2 text-sm font-bold'>
          <input
            type='checkbox'
            checked={f.values.agreeTerms && f.values.agreePrivacy && f.values.over14}
            onChange={(e) => {
              f.set.agreeTerms(e.target.checked)
              f.set.agreePrivacy(e.target.checked)
              f.set.over14(e.target.checked)
            }}
            className='size-[18px] accent-brand'
          />
          전체 동의
        </label>
        <div className='h-px bg-surface-2' />
        <div className='flex items-center gap-2'>
          <div className='grow'>
            <CheckRow checked={f.values.agreeTerms} onChange={f.set.agreeTerms}>
              [필수] 이용약관 동의
            </CheckRow>
          </div>
          <Link to='/terms' target='_blank' className={link}>
            보기
          </Link>
        </div>
        <div className='flex items-center gap-2'>
          <div className='grow'>
            <CheckRow checked={f.values.agreePrivacy} onChange={f.set.agreePrivacy}>
              [필수] 개인정보처리방침 동의
            </CheckRow>
          </div>
          <Link to='/privacy' target='_blank' className={link}>
            보기
          </Link>
        </div>
        <CheckRow checked={f.values.over14} onChange={f.set.over14}>
          [필수] 만 14세 이상이에요
        </CheckRow>
      </Panel>
      <FormAlert message={f.error?.message} />
      <Button type='submit' size='lg' loading={f.pending} disabled={!f.canSubmit} className='w-full'>
        가입하기
      </Button>
      <p className='text-[13px] text-text-muted'>
        이미 계정이 있나요?{' '}
        <Link to='/login' className='font-semibold text-brand hover:underline'>
          로그인
        </Link>{' '}
        · 본인 확인 없이 바로 가입돼요. 비밀번호를 잊으면 복구할 수 없어요.
      </p>
    </form>
  )
}
