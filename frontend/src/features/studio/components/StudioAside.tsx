import type { ReactNode } from 'react'
import { CATEGORY_LABEL } from '../../broadcast/categories.ts'
import ChatCard from '../../chat/components/ChatCard.tsx'
import { Chip } from '../../../components/ui/Chip.tsx'
import Icon from '../../../components/ui/Icon.tsx'
import Panel from '../../../components/ui/Panel.tsx'
import type { useStudio } from '../hooks/useStudio.ts'

type S = ReturnType<typeof useStudio>

function Card({ title, children }: { title: string; children: ReactNode }) {
  return (
    <Panel innerClassName='flex flex-col gap-2.5 p-4'>
      <h2 className='text-[15px] font-bold'>{title}</h2>
      {children}
    </Panel>
  )
}

function Check({ done, children }: { done: boolean; children: ReactNode }) {
  return (
    <li className={`flex items-center gap-2 ${done ? 'text-brand' : 'text-text-muted'}`}>
      {done ? <Icon name='check' className='size-4' /> : <span className='ml-0.5 size-3 rounded-full border border-current' aria-hidden />}
      {children}
    </li>
  )
}

// 송출 전: 시작 전 확인 · 권한 안내 · 지원 환경. 방송 중: 방송 정보(읽기 전용) · 채팅.
export default function StudioAside({ s, live }: { s: S; live: boolean }) {
  const { data, media } = s
  if (!data) return null
  const screen = media.mode === 'screen'
  if (!live)
    return (
      <>
        <Card title='시작 전 확인'>
          <ul className='space-y-2 text-sm'>
            <Check done>로그인됨 · 내 방송</Check>
            <Check done={media.micReady}>{media.micReady ? '마이크 권한 허용' : '마이크 켜기 전'}</Check>
            <Check done={media.videoReady}>
              {screen ? (media.videoReady ? '공유할 화면 선택됨' : '공유할 화면 선택 전') : media.videoReady ? '카메라 켜짐' : '카메라 켜기 전'}
            </Check>
            <Check done={false}>{data.record ? '녹화 켜짐 (최대 3시간)' : '녹화 꺼짐'}</Check>
          </ul>
        </Card>
        <Card title='권한이 막혔을 때'>
          <p className='text-[13px] leading-relaxed text-text-muted'>
            주소창 왼쪽 자물쇠 아이콘 → 사이트 설정 → 카메라·마이크를 &quot;허용&quot;으로 바꾼 뒤 새로고침해 주세요. 화면 공유와 마이크는 HTTPS에서만 동작해요.
          </p>
        </Card>
        <Card title='지원 환경'>
          <p className='text-[13px] leading-relaxed text-text-muted'>
            시스템 소리: Windows Chrome·Edge
            <br />
            탭 소리: 데스크톱 Chrome·Edge
            <br />
            모바일: 카메라 송출 (Android Chrome, iOS Safari)
          </p>
        </Card>
      </>
    )
  return (
    <>
      <Card title='방송 정보'>
        <dl className='space-y-3 text-[13px]'>
          <div>
            <dt className='text-text-muted'>제목</dt>
            <dd className='mt-1 text-sm break-words'>{data.title}</dd>
          </div>
          <div>
            <dt className='text-text-muted'>카테고리</dt>
            <dd className='mt-1 text-sm'>{CATEGORY_LABEL[data.category]}</dd>
          </div>
          {data.tags.length > 0 && (
            <div>
              <dt className='text-text-muted'>태그</dt>
              <dd className='mt-1'>
                <ul className='flex flex-wrap gap-1.5' aria-label='태그'>
                  {data.tags.map((t) => (
                    <li key={t}>
                      <Chip>#{t}</Chip>
                    </li>
                  ))}
                </ul>
              </dd>
            </div>
          )}
        </dl>
        <p className='text-xs text-text-dim'>{data.record ? '방송이 끝나면 다시보기로 남아요.' : '녹화하지 않는 방송이에요.'}</p>
      </Card>
      <ChatCard chat={s.chat} broadcast={data} className='h-96' />
    </>
  )
}
