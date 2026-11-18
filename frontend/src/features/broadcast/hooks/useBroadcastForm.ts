import { useState, type FormEvent, type KeyboardEvent } from 'react'
import { useNavigate } from 'react-router'
import type { Broadcast } from '../../../api/generated/Broadcast.ts'
import type { Category } from '../../../api/generated/Category.ts'
import { useToast } from '../../../hooks/useToast.ts'
import { formError } from '../../auth/hooks/formError.ts'
import { useCreateBroadcast, useMyBroadcasts, useUpdateBroadcast } from './useBroadcasts.ts'
import { READY_LIMIT } from './useManageBroadcasts.ts'

export const MAX_TAGS = 3
export const MAX_TAG_LENGTH = 10
const FIELDS = ['title', 'category', 'tags', 'record']

/** 등록(existing 없음) / 수정(existing 있음) 공용 폼. 상태별로 잠기는 필드는 api.md §3 표를 따른다. */
/** onSaved를 주면 수정 성공 후 이동 대신 그것을 부른다(스튜디오 안 방송 정보 폼). */
export function useBroadcastForm(existing?: Broadcast, onSaved?: () => void) {
  const [title, setTitle] = useState(existing?.title ?? '')
  const [category, setCategory] = useState<Category>(existing?.category ?? 'just_chatting')
  const [tags, setTags] = useState<string[]>(existing?.tags ?? [])
  const [tagDraft, setTagDraft] = useState('')
  const [record, setRecord] = useState(existing?.record ?? true)
  const navigate = useNavigate()
  const toast = useToast()
  const create = useCreateBroadcast()
  const update = useUpdateBroadcast(existing?.id)
  const mine = useMyBroadcasts(!existing)

  const status = existing?.status ?? 'READY'
  const locked = { category: status === 'ENDED', tags: status === 'ENDED', record: status !== 'READY' }
  // 목록은 최신 등록순이라 첫 항목이 가장 최근 방송
  const previous = existing ? undefined : mine.data?.items[0]
  const mutation = existing ? update : create

  // 확정할 태그 목록을 돌려준다(상한·중복 규칙 한 곳)
  const merge = (raw: string[]) => {
    const next = [...tags]
    for (const t of raw) if (t && next.length < MAX_TAGS && !next.includes(t)) next.push(t)
    return next
  }
  const addTags = (raw: string[]) => {
    const next = merge(raw)
    if (next.length > tags.length) setTags(next)
  }
  // 고치는 중에는 이전 서버 오류를 지운다
  const touch = () => {
    if (mutation.isError) mutation.reset()
  }

  const addTag = () => {
    addTags([tagDraft])
    setTagDraft('')
  }

  // 입력칸 앞에 #이 항상 붙어 보이므로 직접 친 #은 걸러낸다. 공백·쉼표가 들어오면 그 앞까지 확정:
  // 한글 조합 중 스페이스는 keydown이 isComposing이라 무시되고 여기로 공백이 들어온다(붙여 넣기도 같은 경로).
  const changeTagDraft = (v: string) => {
    const parts = v.replace(/#/g, '').split(/[\s,]+/)
    const draft = parts.pop() ?? ''
    if (parts.length) addTags(parts)
    setTagDraft(draft)
  }

  // 스페이스·Enter·쉼표로 확정. 한글 조합 중에는 확정하지 않는다(마지막 글자가 깨짐).
  const tagKeyDown = (e: KeyboardEvent<HTMLInputElement>) => {
    if (e.nativeEvent.isComposing) return
    if (e.key === ' ' || e.key === 'Enter' || e.key === ',') {
      e.preventDefault()
      addTag()
    } else if (e.key === 'Backspace' && !tagDraft && tags.length > 0) {
      setTags(tags.slice(0, -1))
    }
  }

  const copyPrevious = () => {
    if (!previous) return
    setTitle(previous.title)
    setCategory(previous.category)
    setTags(previous.tags)
    setRecord(previous.record)
  }

  const done = (to: string) => {
    toast({ tone: 'success', message: existing ? '방송 정보를 저장했어요.' : '방송을 등록했어요.' })
    void navigate(to)
  }

  // 등록은 기본이 스튜디오로 이동(시안), 'manage'면 대기 방송으로 저장하고 관리로 간다
  const send = (dest: 'studio' | 'manage') => {
    const finalTags = merge([tagDraft]) // 확정 안 한 태그 입력도 버리지 않는다
    const base = { title: title.trim(), category, tags: finalTags, record }
    if (!existing)
      return create.mutate(base, { onSuccess: (b) => done(dest === 'studio' ? `/studio?broadcast=${b.id}` : '/manage') })
    // 잠긴 필드는 null(= 변경 없음)로 보낸다. 보내면 서버가 409로 거절한다.
    update.mutate(
      {
        title: base.title,
        category: locked.category ? null : category,
        tags: locked.tags ? null : finalTags,
        record: locked.record ? null : record,
        pipCorner: null,
      },
      { onSuccess: () => (onSaved ? (toast({ tone: 'success', message: '방송 정보를 저장했어요.' }), onSaved()) : done('/manage')) },
    )
  }

  const submit = (e: FormEvent) => {
    e.preventDefault()
    send('studio')
  }

  const readyCount = mine.data?.items.filter((b) => b.status === 'READY').length ?? 0

  return {
    values: { title, category, tags, tagDraft, record },
    set: {
      title: (v: string) => (touch(), setTitle(v)),
      category: (v: Category) => (touch(), setCategory(v)),
      tagDraft: (v: string) => (touch(), changeTagDraft(v)),
      record: setRecord,
    },
    saveToManage: () => send('manage'),
    full: !existing && readyCount >= READY_LIMIT,
    removeTag: (tag: string) => setTags(tags.filter((t) => t !== tag)),
    tagKeyDown,
    copyPrevious,
    canCopy: !!previous,
    readyCount,
    locked,
    status,
    pending: mutation.isPending,
    error: mutation.isError ? formError(mutation.error, FIELDS) : null,
    submit,
  }
}
