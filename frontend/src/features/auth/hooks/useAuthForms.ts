import { useState, type FormEvent } from 'react'
import { ApiRequestError } from '../../../api/client.ts'
import type { TestLoginRequest } from '../../../api/generated/TestLoginRequest.ts'
import { useToast } from '../../../hooks/useToast.ts'
import { formError } from './formError.ts'
import { useChangePassword, useLogin, useMe, useSignup, useTestLogin, useUpdateNickname } from './useAuth.ts'

const prevent = (fn: () => void) => (e: FormEvent) => {
  e.preventDefault()
  fn()
}

export function useLoginForm() {
  const [loginId, setLoginId] = useState('')
  const [password, setPassword] = useState('')
  const [remember, setRemember] = useState(false)
  const login = useLogin()
  const test = useTestLogin()
  const failed = login.isError ? login.error : test.isError ? test.error : null
  return {
    values: { loginId, password, remember },
    set: { loginId: setLoginId, password: setPassword, remember: setRemember },
    pending: login.isPending || test.isPending,
    error: failed ? formError(failed, ['loginId', 'password']) : null,
    submit: prevent(() => login.mutate({ loginId: loginId.trim(), password, remember })),
    testLogin: (account: TestLoginRequest['account']) => test.mutate({ account }),
  }
}

export function useSignupForm() {
  const [loginId, setLoginId] = useState('')
  const [password, setPassword] = useState('')
  const [nickname, setNickname] = useState('')
  const [agreeTerms, setAgreeTerms] = useState(false)
  const [agreePrivacy, setAgreePrivacy] = useState(false)
  const [over14, setOver14] = useState(false)
  const signup = useSignup()
  return {
    values: { loginId, password, nickname, agreeTerms, agreePrivacy, over14 },
    set: { loginId: setLoginId, password: setPassword, nickname: setNickname, agreeTerms: setAgreeTerms, agreePrivacy: setAgreePrivacy, over14: setOver14 },
    pending: signup.isPending,
    canSubmit: agreeTerms && agreePrivacy && over14,
    error: signup.isError ? formError(signup.error, ['loginId', 'password', 'nickname']) : null,
    submit: prevent(() => signup.mutate({ loginId: loginId.trim(), password, nickname: nickname.trim(), agreeTerms, agreePrivacy, over14 })),
  }
}

const DAY = 86_400_000

export function useNicknameForm() {
  const { data: me } = useMe()
  const [draft, setDraft] = useState<string | null>(null)
  const [now] = useState(Date.now)
  const toast = useToast()
  const update = useUpdateNickname()
  const nickname = draft ?? me?.nickname ?? ''
  // 서버 시각 기준 변경 가능 시각까지 남은 일수(올림). 0이면 지금 바꿀 수 있다.
  const left = me?.nicknameChangeableAt ? new Date(me.nicknameChangeableAt).getTime() - now : 0
  // 방금 바꾼 직후에 31일로 보이지 않게 1분 여유를 두되, 잠겨 있으면 최소 1일
  const daysLeft = left > 0 ? Math.max(1, Math.ceil((left - 60_000) / DAY)) : 0
  return {
    nickname,
    setNickname: setDraft,
    daysLeft,
    pending: update.isPending,
    error: update.isError ? formError(update.error, ['nickname']) : null,
    submit: prevent(() =>
      update.mutate(nickname.trim(), {
        onSuccess: () => {
          setDraft(null)
          toast({ tone: 'success', message: '닉네임을 바꿨어요.' })
        },
      }),
    ),
  }
}

export function usePasswordForm() {
  const [currentPassword, setCurrent] = useState('')
  const [newPassword, setNew] = useState('')
  const toast = useToast()
  const change = useChangePassword()
  const error = change.isError ? formError(change.error, ['currentPassword', 'newPassword']) : null
  // 현재 비밀번호가 틀린 경우는 필드 아래에 보여 준다
  const wrong = change.error instanceof ApiRequestError && change.error.code === 'AUTH_WRONG_PASSWORD'
  return {
    values: { currentPassword, newPassword },
    set: { currentPassword: setCurrent, newPassword: setNew },
    pending: change.isPending,
    error: wrong && error ? { fields: { ...error.fields, currentPassword: error.message ?? '' }, message: undefined } : error,
    submit: prevent(() =>
      change.mutate(
        { currentPassword, newPassword },
        {
          onSuccess: () => {
            setCurrent('')
            setNew('')
            toast({ tone: 'success', message: '비밀번호를 바꿨어요. 다른 기기에서는 로그아웃돼요.' })
          },
        },
      ),
    ),
  }
}
