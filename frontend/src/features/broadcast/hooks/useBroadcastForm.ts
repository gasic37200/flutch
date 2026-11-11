import { useState, type FormEvent, type KeyboardEvent } from 'react'
import { useNavigate } from 'react-router'
import type { Broadcast } from '../../../api/generated/Broadcast.ts'
import type { Category } from '../../../api/generated/Category.ts'
import { useToast } from '../../../hooks/useToast.ts'
import { formError } from '../../auth/hooks/formError.ts'
import { useCreateBroadcast, useMyBroadcasts, useUpdateBroadcast } from './useBroadcasts.ts'

export const MAX_TAGS = 3
export const MAX_TAG_LENGTH = 10
const FIELDS = ['title', 'category', 'tags', 'record']

/** 등록(existing 없음) / 수정(existing 있음) 공용 폼. 상태별로 잠기는 필드는 api.md §3 표를 따른다. */
export function useBroadcastForm(existing?: Broadcast) {
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

  const addTag = () => {
    const tag = tagDraft
    setTagDraft('')
    if (tag && tags.length < MAX_TAGS && !tags.includes(tag)) setTags([...tags, tag])
  }

  // 입력칸 앞에 #이 항상 붙어 보이므로 직접 친 #·공백은 걸러낸다
  const changeTagDraft = (v: string) => setTagDraft(v.replace(/[#\s]/g, ''))

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

  const done = () => {
    toast({ tone: 'success', message: existing ? '방송 정보를 저장했어요.' : '방송을 등록했어요. 내 방송 관리에서 시작할 수 있어요.' })
    void navigate('/manage')
  }

  const submit = (e: FormEvent) => {
    e.preventDefault()
    const base = { title: title.trim(), category, tags, record }
    if (!existing) return create.mutate(base, { onSuccess: done })
    // 잠긴 필드는 null(= 변경 없음)로 보낸다. 보내면 서버가 409로 거절한다.
    update.mutate(
      {
        title: base.title,
        category: locked.category ? null : category,
        tags: locked.tags ? null : tags,
        record: locked.record ? null : record,
        pipCorner: null,
      },
      { onSuccess: done },
    )
  }

  return {
    values: { title, category, tags, tagDraft, record },
    set: { title: setTitle, category: setCategory, tagDraft: changeTagDraft, record: setRecord },
    removeTag: (tag: string) => setTags(tags.filter((t) => t !== tag)),
    tagKeyDown,
    copyPrevious,
    canCopy: !!previous,
    readyCount: mine.data?.items.filter((b) => b.status === 'READY').length ?? 0,
    locked,
    status,
    pending: mutation.isPending,
    error: mutation.isError ? formError(mutation.error, FIELDS) : null,
    submit,
  }
}
