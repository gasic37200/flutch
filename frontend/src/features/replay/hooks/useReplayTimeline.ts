import { useEffect, useMemo } from 'react'
import { useDonationList } from '../../donation/hooks/useDonationList.ts'
import { buildTimeline } from '../timeline.ts'
import { useReplayChats } from './useRecordings.ts'

/** 한 방송의 채팅·후원을 전부 받아(페이지가 남아 있으면 이어서) 영상 시점순 타임라인으로 만든다. shortcut: 전량 로드라 채팅이 수만 건이면 느리다, 그때 구간 조회 API를 둔다. */
export function useReplayTimeline(broadcastId: number | undefined, startedAt: string | undefined) {
  const chats = useReplayChats(broadcastId)
  const don = useDonationList(broadcastId)
  const { hasNextPage: moreChats, isFetchingNextPage: fetchingChats, fetchNextPage: nextChats } = chats
  const { hasNextPage: moreDon, isFetchingNextPage: fetchingDon, fetchNextPage: nextDon } = don.query
  useEffect(() => {
    if (moreChats && !fetchingChats) void nextChats()
  }, [moreChats, fetchingChats, nextChats])
  useEffect(() => {
    if (moreDon && !fetchingDon) void nextDon()
  }, [moreDon, fetchingDon, nextDon])
  const chatData = chats.data
  const donData = don.query.data
  const items = useMemo(() => {
    if (!startedAt) return []
    const c = chatData ? [...chatData.pages].reverse().flatMap((p) => p.items) : []
    return buildTimeline(c, donData?.pages.flatMap((p) => p.items) ?? [], startedAt)
  }, [chatData, donData, startedAt])
  return {
    items,
    loading: chats.isPending || don.query.isPending || !!moreChats || !!moreDon,
    error: chats.isError || don.query.isError,
    donationCount: don.count,
  }
}
